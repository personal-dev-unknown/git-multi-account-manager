/* Git Manager — global JS */
document.addEventListener('DOMContentLoaded', () => {
  // Auto-connect SSE for real-time updates
  if (window.EventSource) {
    const source = new EventSource('/api/sse');
    source.onerror = () => { /* reconnect handled by browser */ };
  }
  // Handle form confirmation dialogs
  document.querySelectorAll('[data-confirm]').forEach(el => {
    el.addEventListener('click', e => {
      if (!confirm(el.dataset.confirm)) e.preventDefault();
    });
  });
});
