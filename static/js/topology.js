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
                password: node.password
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
                    'background-color': '#4a90e2',
                    'background-image': 'data:image/svg+xml;base64,PHN2ZyB3aWR0aD0iMjQiIGhlaWdodD0iMjQiIHZpZXdCb3g9IjAgMCAyNCAyNCIgZmlsbD0ibm9uZSIgeG1sbnM9Imh0dHA6Ly93d3cudzMub3JnLzIwMDAvc3ZnIj48cGF0aCBkPSJNMiA0QzIgMi44OTU0MyAyLjg5NTQzIDIgNCAySDIwQzIxLjEwNDYgMiAyMiAyLjg5NTQzIDIyIDRWMjBDMjIgMjEuMTA0NiAyMS4xMDQ2IDIyIDIwIDIySDRDMi44OTU0MyAyMiAyIDIxLjEwNDYgMiAyMFY0Wk00IDRWMjBIMjBWNEg0Wk02IDdIOFY5SDZWMTFIOEgxMFYxM0g2VjE1SDhWMTdINlYxOEg4VjIwSDEwVjE4SDEyVjIwSDE0VjE4SDE2VjIwSDE4VjE4SDIwVjE2SDE4VjE0SDIwVjEySDE4VjEwSDE2VjhIMTRWMTBIMTJWOFYxMEgxMFY4SDhWMTBWN0g2WiIgZmlsbD0id2hpdGUiLz48L3N2Zz4=',
                    'background-fit': 'contain',
                    'background-clip': 'node',
                    'background-padding': '8px',
                    'label': 'data(name)',
                    'color': '#fff',
                    'text-valign': 'bottom',
                    'text-halign': 'center',
                    'width': '60px',
                    'height': '70px',
                    'font-size': '11px',
                    'font-weight': 'bold',
                    'text-margin-y': '5px',
                    'border-width': '2px',
                    'border-color': '#2c5f8d',
                    'border-opacity': '0.8'
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

    // Auto-refresh every 30 seconds
    setTimeout(loadTopology, 30000);
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
