/* Server-Sent Events client — listens for kernel events and refreshes UI */
class SSEClient {
  constructor(url = '/api/sse') {
    this.url = url; this.handlers = {};
    this.connect();
  }
  connect() {
    this.source = new EventSource(this.url);
    this.source.onmessage = (e) => {
      try {
        const event = JSON.parse(e.data);
        const h = this.handlers[event.event_type];
        if (h) h(event);
        const any = this.handlers['*'];
        if (any) any(event);
      } catch (_) {}
    };
    this.source.onerror = () => setTimeout(() => this.connect(), 3000);
  }
  on(eventType, handler) { this.handlers[eventType] = handler; return this; }
}
window.sseClient = new SSEClient();
