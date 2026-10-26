use crate::config::Config;
use crate::repository::ServerRepository;
use std::net::ToSocketAddrs;
use std::sync::Arc;
use std::time::Duration;
use tokio::time::{interval, sleep};

pub struct HealthCheckService {
    server_repo: Arc<ServerRepository>,
    config: Arc<Config>,
}

impl HealthCheckService {
    pub fn new(server_repo: Arc<ServerRepository>, config: Arc<Config>) -> Self {
        Self { server_repo, config }
    }

    pub async fn run(&self) {
        let mut interval = interval(Duration::from_secs(60));

        loop {
            interval.tick().await;
            self.check_all_servers().await;
        }
    }

    async fn check_all_servers(&self) {
        let servers = match self.server_repo.get_all() {
            Ok(s) => s,
            Err(e) => {
                tracing::error!("Failed to get servers for health check: {}", e);
                return;
            }
        };

        for server in servers {
            let status = self.check_server(&server.hostname, server.port as u16).await;
            tracing::debug!("Health check for {}: {}", server.name, status);

            // Determine if server is on intranet (private IP)
            let is_intranet = server.hostname.starts_with("10.")
                || server.hostname.starts_with("172.16.")
                || server.hostname.starts_with("192.168.")
                || server.hostname == "localhost";

            // Update server status in database
            if let Err(e) = self.server_repo.update_status(server.id, &status, is_intranet) {
                tracing::error!("Failed to update status for server {}: {}", server.name, e);
            }
        }
    }

    async fn check_server(&self, hostname: &str, port: u16) -> String {
        match tokio::time::timeout(
            Duration::from_secs(5),
            tokio::net::TcpStream::connect((hostname, port)),
        )
        .await
        {
            Ok(Ok(_)) => "online".to_string(),
            _ => "offline".to_string(),
        }
    }
}