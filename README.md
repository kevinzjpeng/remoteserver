# Server Access Manager

A web-based server access management system that stores SSH credentials, provides GitHub OAuth authentication, includes an admin panel for credential management, and generates topology diagrams showing server relationships by environment.

## Features

- SSH credential storage with encryption
- GitHub OAuth authentication
- Admin panel for managing credentials
- Auto-generated topology diagrams
- Single binary deployment

## Setup

1. Clone the repository
2. Copy `.env.example` to `.env` and configure
3. Run `go run cmd/server/main.go`
4. Visit `http://localhost:8080`

## Configuration

See `.env.example` for all configuration options.

## Development

```bash
# Run tests
go test ./...

# Run the application
go run cmd/server/main.go
```

## Building

```bash
go build -o server-access-manager cmd/server/main.go
```

## Deployment

The application compiles to a single binary. Set environment variables and run:

```bash
./server-access-manager
```
