// src/git_manager/web/static/js/main.js
// Main JavaScript

const socket = io();

// Socket.IO event handlers
socket.on('connect', () => {
    console.log('Connected to server');
    showNotification('Connected', 'success');
});

socket.on('disconnect', () => {
    console.log('Disconnected from server');
    showNotification('Disconnected', 'warning');
});

socket.on('clone_progress', (data) => {
    updateCloneProgress(data);
});

// API helper functions
async function apiRequest(url, method = 'GET', data = null) {
    const options = {
        method,
        headers: {
            'Content-Type': 'application/json'
        }
    };
    
    if (data) {
        options.body = JSON.stringify(data);
    }
    
    try {
        const response = await fetch(url, options);
        const result = await response.json();
        
        if (!response.ok) {
            throw new Error(result.error || 'Request failed');
        }
        
        return result;
    } catch (error) {
        showNotification(error.message, 'error');
        throw error;
    }
}

// Notification system
function showNotification(message, type = 'info') {
    const notification = document.createElement('div');
    notification.className = `alert alert-${type}`;
    notification.textContent = message;
    
    document.body.appendChild(notification);
    
    setTimeout(() => {
        notification.remove();
    }, 5000);
}

// Clone progress update
function updateCloneProgress(data) {
    const progressDiv = document.getElementById('clone-progress');
    if (!progressDiv) return;
    
    if (data.status === 'starting') {
        progressDiv.innerHTML = `<div class="spinner"></div><p>Cloning ${data.url}...</p>`;
    } else if (data.status === 'complete') {
        progressDiv.innerHTML = `<p class="alert alert-success">Cloned to ${data.path}</p>`;
    } else if (data.status === 'error') {
        progressDiv.innerHTML = `<p class="alert alert-error">Error: ${data.error}</p>`;
    }
}

// Load accounts
async function loadAccounts() {
    const accounts = await apiRequest('/api/v1/accounts');
    displayAccounts(accounts);
}

// Display accounts in table
function displayAccounts(accounts) {
    const tbody = document.querySelector('#accounts-table tbody');
    if (!tbody) return;
    
    tbody.innerHTML = '';
    
    accounts.forEach(account => {
        const row = document.createElement('tr');
        row.innerHTML = `
            <td>${account.name}</td>
            <td>${account.platform}</td>
            <td>${account.username}</td>
            <td>${account.email}</td>
            <td>
                <button class="btn btn-primary" onclick="testAccount('${account.name}')">Test</button>
                <button class="btn btn-danger" onclick="deleteAccount('${account.name}')">Delete</button>
            </td>
        `;
        tbody.appendChild(row);
    });
}

// Test account
async function testAccount(name) {
    showNotification(`Testing ${name}...`, 'info');
    const result = await apiRequest(`/api/v1/accounts/${name}/test`, 'POST');
    
    if (result.success) {
        showNotification('Connection successful!', 'success');
    } else {
        showNotification('Connection failed: ' + result.message, 'error');
    }
}

// Delete account
async function deleteAccount(name) {
    if (!confirm(`Delete account ${name}?`)) return;
    
    await apiRequest(`/api/v1/accounts/${name}`, 'DELETE');
    showNotification(`Account ${name} deleted`, 'success');
    loadAccounts();
}

// Initialize on page load
document.addEventListener('DOMContentLoaded', () => {
    // Load data based on current page
    if (document.getElementById('accounts-table')) {
        loadAccounts();
    }
});