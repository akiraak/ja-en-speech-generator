import { invoke } from "@tauri-apps/api/tauri";
import { listen } from '@tauri-apps/api/event';

async function setup() {
  const commandInputEl = document.querySelector<HTMLInputElement>("#command-input");
  const outputEl = document.querySelector<HTMLElement>("#output");

  const addOutput = (text: string) => {
    if (outputEl) {
      outputEl.textContent += `\n${text}`;
      outputEl.scrollTop = outputEl.scrollHeight;
    }
  };

  const submitCommand = async () => {
    if (!commandInputEl || !outputEl) return;

    const command = commandInputEl.value;
    commandInputEl.value = "";
    const response = await invoke("submit_command", {command: command});
    addOutput(response as string);
  };

  document.querySelector<HTMLFormElement>("#command-form")?.addEventListener("submit", (e) => {
    e.preventDefault();
    submitCommand();
  });

  listen('add_to_output', event => {
    addOutput(event.payload as string);
  });
}

window.addEventListener("DOMContentLoaded", setup);
