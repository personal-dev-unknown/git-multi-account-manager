// src/git_manager/web/static/js/ssh.js
// SSH Key Management

// Show add to agent modal
function showAddToAgentModal(keyPath) {
    const modal = document.getElementById('addToAgentModal');
    const keyPathInput = document.getElementById('keyPath');
    keyPathInput.value = keyPath;
    modal.style.display = 'block';
}

// Close add to agent modal
function closeAddToAgentModal() {
    const modal = document.getElementById('addToAgentModal');
    modal.style.display = 'none';
}

// Add key to SSH agent
async function addKeyToAgent() {
    const keyPath = document.getElementById('keyPath').value;
    const passphrase = document.getElementById('passphrase').value;
    const submitBtn = document.getElementById('addToAgentBtn');
    const loadingText = document.getElementById('addToAgentLoading');
    
    submitBtn.disabled = true;
    loadingText.style.display = 'inline';
    
    try {
        const response = await fetch('/api/v1/ssh/add-to-agent', {
            method: 'POST',
            headers: {
                'Content-Type': 'application/json',
            },
            body: JSON.stringify({
                key_path: keyPath,
                passphrase: passphrase || undefined
            })
        });
        
        const result = await response.json();
        
        if (!response.ok) {
            throw new Error(result.error || 'Failed to add key to agent');
        }
        
        showNotification('Key added to SSH agent successfully', 'success');
        closeAddToAgentModal();
    } catch (error) {
        showNotification(error.message, 'error');
    } finally {
        submitBtn.disabled = false;
        loadingText.style.display = 'none';
    }
}

// Load SSH keys
async function loadSSHKeys() {
    try {
        const response = await fetch('/api/v1/ssh/keys');
        const keys = await response.json();
        displaySSHKeys(keys);
    } catch (error) {
        showNotification('Failed to load SSH keys: ' + error.message, 'error');
    }
}

// Display SSH keys in the table
function displaySSHKeys(keys) {
    const tbody = document.querySelector('#ssh-keys-table tbody');
    tbody.innerHTML = '';
    
    keys.forEach(key => {
        const row = document.createElement('tr');
        
        // Format created date
        const createdDate = new Date(key.created_at).toLocaleString();
        
        // Create action buttons
        const actions = [
            `<button class="btn btn-sm btn-primary" onclick="showAddToAgentModal('${key.private_key_path}')">
                Add to Agent
            </button>`,
            `<button class="btn btn-sm btn-secondary" onclick="copyToClipboard('${key.public_key}')">
                Copy Public Key
            </button>`,
            `<button class="btn btn-sm btn-danger" onclick="deleteSSHKey('${key.name}')">
                Delete
            </button>`
        ].join(' ');
        
        // Create table row
        row.innerHTML = `
            <td>${key.name}</td>
            <td>${key.type}</td>
            <td>${key.private_key_path}</td>
            <td>${createdDate}</td>
            <td class="actions">${actions}</td>
        `;
        
        tbody.appendChild(row);
    });
}

// Copy text to clipboard
function copyToClipboard(text) {
    navigator.clipboard.writeText(text).then(() => {
        showNotification('Copied to clipboard', 'success');
    }).catch(err => {
        showNotification('Failed to copy: ' + err, 'error');
    });
}

// Delete SSH key
async function deleteSSHKey(name) {
    if (!confirm(`Are you sure you want to delete the SSH key "${name}"?`)) {
        return;
    }
    
    try {
        const response = await fetch(`/api/v1/ssh/keys/${encodeURIComponent(name)}`, {
            method: 'DELETE'
        });
        
        if (!response.ok) {
            const result = await response.json();
            throw new Error(result.error || 'Failed to delete SSH key');
        }
        
        showNotification('SSH key deleted successfully', 'success');
        loadSSHKeys(); // Refresh the list
    } catch (error) {
        showNotification(error.message, 'error');
    }
}

// Initialize SSH key management when the page loads
document.addEventListener('DOMContentLoaded', () => {
    if (document.getElementById('ssh-keys-table')) {
        loadSSHKeys();
    }
});
