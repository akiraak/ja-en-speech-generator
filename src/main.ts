import { invoke } from "@tauri-apps/api/tauri";
import { listen } from '@tauri-apps/api/event';

async function setup() {
  const commandInputEl = document.querySelector<HTMLInputElement>("#command-input");
  const outputEl = document.querySelector<HTMLElement>("#output");

  const submitCommand = async () => {
    if (!commandInputEl || !outputEl) return;

    const command = commandInputEl.value;
    commandInputEl.value = "";
    const response = await invoke("submit_command", {command: command});
    outputEl.textContent += `\n${response}`;
  };

  document.querySelector<HTMLFormElement>("#command-form")?.addEventListener("submit", (e) => {
    e.preventDefault();
    submitCommand();
  });

  listen('add_to_output', event => {
    if (outputEl) {
      outputEl.textContent += `\n${event.payload}`;
    }
  });
}

window.addEventListener("DOMContentLoaded", setup);
