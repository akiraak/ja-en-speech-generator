/*
import { invoke } from "@tauri-apps/api/tauri";
import { listen } from '@tauri-apps/api/event';

let commandInputEl: HTMLInputElement | null;
let outputEl: HTMLElement | null;

async function submit_command() {
  if (commandInputEl && outputEl) {
    let command = commandInputEl.value;
    commandInputEl.value = "";
    let response = await invoke("submit_command", {
      command: command,
    });
    //let response = await invoke("init_process", {
    //  command: command,
    //});
    outputEl.textContent += "\n" + response;
  }
}

async function handleEventAddToOutput(event: any) {
  console.log('Data received from Rust:', event.payload);
  if (outputEl) {
    outputEl.textContent += "\n" + event.payload;
  }
}

window.addEventListener("DOMContentLoaded", () => {
  commandInputEl = document.querySelector("#command-input");
  outputEl = document.querySelector("#output");
  document.querySelector("#command-form")?.addEventListener("submit", (e) => {
    e.preventDefault();
    submit_command();
  });

  // イベントリスナーを設定
  listen('add_to_output', handleEventAddToOutput);
});
*/

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
    //const response = await invoke("init_process", {command: command});
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
