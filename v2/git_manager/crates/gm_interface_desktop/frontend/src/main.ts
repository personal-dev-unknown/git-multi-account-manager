/**
 * Svelte application entry point.
 *
 * Imports the shared design system CSS (variables, components, utilities)
 * then mounts the root App component onto the #app div defined in index.html.
 *
 * Tauri's IPC bridge is automatically available after window load —
 * no explicit initialisation is needed here.
 */

import './app.css';
import App from './App.svelte';

const app = new App({
  target: document.getElementById('app') as HTMLElement,
});

export default app;