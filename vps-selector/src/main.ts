import { createApp } from './app';

window.addEventListener("DOMContentLoaded", () => {
  const root = document.querySelector<HTMLElement>(".container") ?? document.body;
  createApp(root);
});
