let greetInputEl: HTMLInputElement | null;
let greetMsgEl: HTMLElement | null;

function showPlaceholder() {
  if (greetMsgEl && greetInputEl) {
    greetMsgEl.textContent = greetInputEl.value.trim()
      ? `Hello, ${greetInputEl.value.trim()}!`
      : "Hello!";
  }
}

window.addEventListener("DOMContentLoaded", () => {
  greetInputEl = document.querySelector("#greet-input");
  greetMsgEl = document.querySelector("#greet-msg");
  document.querySelector("#greet-form")?.addEventListener("submit", (e) => {
    e.preventDefault();
    showPlaceholder();
  });
});
