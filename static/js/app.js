// App JavaScript
document.body.addEventListener('htmx:configRequest', (event) => {
    // Add CSRF token to all HTMX requests
    const csrfToken = document.querySelector('meta[name="csrf-token"]');
    if (csrfToken) {
        event.detail.headers['X-CSRF-Token'] = csrfToken.getAttribute('content');
    }
});

document.body.addEventListener('htmx:beforeSwap', (event) => {
    // Handle redirects
    if (event.detail.xhr.status === 302 || event.detail.xhr.status === 303) {
        window.location.href = event.detail.xhr.getResponseHeader('Location');
    }
});

// Show modal
function showModal() {
    document.getElementById('modal').style.display = 'block';
}

// Hide modal
function hideModal() {
    document.getElementById('modal').style.display = 'none';
}

// Close modal on outside click
document.addEventListener('click', (event) => {
    const modal = document.getElementById('modal');
    if (modal && event.target === modal) {
        hideModal();
    }
});
