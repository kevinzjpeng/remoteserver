// Topology Visualization using Cytoscape.js
document.addEventListener('DOMContentLoaded', function() {
    loadTopology();
});

function loadTopology() {
    fetch('/api/topology')
        .then(response => response.json())
        .then(data => {
            renderTopology(data);
        })
        .catch(error => {
            console.error('Error loading topology:', error);
            document.getElementById('topology-container').innerHTML =
                '<p style="color: red;">Failed to load topology</p>';
        });
}

function renderTopology(data) {
    const container = document.getElementById('topology-container');

    // Prepare elements for Cytoscape
    const elements = [];

    // Default to empty arrays if data is null
    const groups = data.groups || [];
    const nodes = data.nodes || [];
    const edges = data.edges || [];

    // Add compound nodes for environment groups
    groups.forEach(group => {
        elements.push({
            data: {
                id: 'group-' + group.id,
                name: group.name,
                color: group.color
            }
        });
    });

    // Add server nodes
    nodes.forEach(node => {
        const connectionInfo = node.username + '@' + node.hostname + (node.port && node.port !== 22 ? ':' + node.port : '');
        elements.push({
            data: {
                id: 'node-' + node.id,
                name: node.name,
                parent: 'group-' + node.environment,
                type: node.type,
                status: node.status,
                username: node.username,
                hostname: node.hostname,
                port: node.port,
                password: node.password,
                displayName: node.name,
                displayInfo: connectionInfo
            }
        });
    });

    // Add edges
    edges.forEach(edge => {
        elements.push({
            data: {
                id: 'edge-' + edge.source + '-' + edge.target,
                source: 'node-' + edge.source,
                target: 'node-' + edge.target,
                label: edge.label || ''
            }
        });
    });

    // Initialize Cytoscape
    const cy = cytoscape({
        container: container,
        elements: elements,
        style: [
            {
                selector: 'node[parent^="group-"]',
                style: {
                    'shape': 'rectangle',
                    'background-color': '#2c5f8d',
                    'background-image': 'data:image/svg+xml;base64,PHN2ZyB3aWR0aD0iNjQiIGhlaWdodD0iNjQiIHZpZXdCb3g9IjAgMCA2NCA2NCIgZmlsbD0ibm9uZSIgeG1sbnM9Imh0dHA6Ly93d3cudzMub3JnLzIwMDAvc3ZnIj4KPHJlY3QgeD0iOCIgeT0iNCIgd2lkdGg9IjQ4IiBoZWlnaHQ9IjU2IiByeD0iNCIgZmlsbD0iIzNkN2FjMiIgc3Ryb2tlPSIjMWM1ZDkwIiBzdHJva2Utd2lkdGg9IjIiLz4KPHJlY3QgeD0iMTIiIHk9IjgiIHdpZHRoPSI0MCIgaGVpZ2h0PSIxMiIgcng9IjIiIGZpbGw9IiMxYzVkOTAiLz4KPGNpcmNsZSBjeD0iMTYiIGN5PSIxNCIgcj0iMiIgZmlsbD0iIzRkYWFlYSIvPgo8Y2lyY2xlIGN4PSIyMiIgY3k9IjE0IiByPSIyIiBmaWxsPSIjNGRhYWVhIi8+CjxjaXJjbGUgY3g9IjI4IiBjeT0iMTQiIHI9IjIiIGZpbGw9IiM0ZGFhZWEiLz4KPHJlY3QgeD0iMTIiIHk9IjI0IiB3aWR0aD0iNDAiIGhlaWdodD0iMTIiIHJ4PSIyIiBmaWxsPSIjMWM1ZDkwIi8+CjxjaXJjbGUgY3g9IjE2IiBjeT0iMzAiIHI9IjIiIGZpbGw9IiM0ZGFhZWEiLz4KPGNpcmNsZSBjeD0iMjIiIGN5PSIzMCIgcj0iMiIgZmlsbD0iIzRkYWFlYSIvPgo8Y2lyY2xlIGN4PSIyOCIgY3k9IjMwIiByPSIyIiBmaWxsPSIjNGRhYWVhIi8+CjxyZWN0IHg9IjEyIiB5PSI0MCIgd2lkdGg9IjQwIiBoZWlnaHQ9IjEyIiByeD0iMiIgZmlsbD0iIzFjNWQ5MCIvPgo8Y2lyY2xlIGN4PSIxNiIgY3k9IjQ2IiByPSIyIiBmaWxsPSIjNGRhYWVhIi8+CjxjaXJjbGUgY3g9IjIyIiBjeT0iNDYiIHI9IjIiIGZpbGw9IiM0ZGFhZWEiLz4KPGNpcmNsZSBjeD0iMjgiIGN5PSI0NiIgcj0iMiIgZmlsbD0iIzRkYWFlYSIvPgo8L3N2Zz4K',
                    'background-fit': 'contain',
                    'background-clip': 'node',
                    'background-padding': '4px',
                    'label': 'data(displayName)',
                    'color': '#fff',
                    'text-valign': 'bottom',
                    'text-halign': 'center',
                    'width': '70px',
                    'height': '80px',
                    'font-size': '12px',
                    'font-weight': 'bold',
                    'text-margin-y': '20px',
                    'border-width': '3px',
                    'border-color': '#1c5d90',
                    'border-opacity': '1',
                    'shadow-blur': '10',
                    'shadow-color': '#000',
                    'shadow-opacity': '0.3',
                    'shadow-offset-x': '2',
                    'shadow-offset-y': '2'
                }
            },
            {
                selector: 'node[^parent]',
                style: {
                    'background-color': '#999',
                    'label': 'data(name)',
                    'color': '#fff',
                    'text-valign': 'center',
                    'text-halign': 'center',
                    'font-size': '12px',
                    'font-weight': 'bold',
                    'text-opacity': 0.7
                }
            },
            {
                selector: 'edge',
                style: {
                    'width': 2,
                    'line-color': '#ccc',
                    'target-arrow-color': '#ccc',
                    'target-arrow-shape': 'triangle',
                    'curve-style': 'bezier',
                    'label': 'data(label)',
                    'font-size': '8px',
                    'text-opacity': 0.7
                }
            }
        ],
        layout: {
            name: 'cose',
            padding: 10,
            nodeOverlap: 20,
            refresh: 20,
            fit: true,
            animate: true,
            animationDuration: 500
        }
    });

    // Create HTML overlays for connection info (smaller font)
    function updateConnectionLabels() {
        // Remove existing labels
        document.querySelectorAll('.connection-label').forEach(el => el.remove());

        // Add new labels for each server node
        cy.nodes('[parent^="group-"]').forEach(node => {
            const data = node.data();
            if (data.displayInfo) {
                const pos = node.renderedPosition();
                const label = document.createElement('div');
                label.className = 'connection-label';
                label.textContent = data.displayInfo;
                label.style.cssText = `
                    position: absolute;
                    left: ${pos.x}px;
                    top: ${pos.y + 50}px;
                    transform: translateX(-50%);
                    font-size: 8px;
                    color: #aaa;
                    text-align: center;
                    pointer-events: none;
                    white-space: nowrap;
                    z-index: 10;
                `;
                container.appendChild(label);
            }
        });
    }

    // Update labels on initial render and when layout changes
    cy.on('render', updateConnectionLabels);
    updateConnectionLabels();

    // Add click handlers
    cy.on('tap', 'node[parent^="group-"]', function(evt) {
        const node = evt.target;
        const data = node.data();

        // If the node has connection details, open SSH terminal
        if (data.username && data.hostname) {
            let sshUrl = `ssh://`;
            sshUrl += encodeURIComponent(data.username);
            if (data.password) {
                sshUrl += `:${encodeURIComponent(data.password)}`;
            }
            sshUrl += `@${data.hostname}`;
            if (data.port && data.port !== 22) {
                sshUrl += `:${data.port}`;
            }
            window.open(sshUrl, '_blank');
        } else {
            // Otherwise show details
            showNodeDetails(data);
        }
    });
}

function showNodeDetails(data) {
    const detailsPanel = document.getElementById('node-details');
    if (detailsPanel) {
        detailsPanel.innerHTML = `
            <h3>${data.name}</h3>
            <p><strong>Type:</strong> ${data.type || 'Unknown'}</p>
            <p><strong>Status:</strong> ${data.status || 'Unknown'}</p>
        `;
    }
}
