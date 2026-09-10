import { invoke } from "@tauri-apps/api/core";

interface ConversionResult {
  unixtime: string | null;
  message: string;
}

function el<T extends HTMLElement>(id: string): T {
  const found = document.getElementById(id);
  if (!found) throw new Error(`#${id} not found`);
  return found as T;
}

const statusMessage = () => el<HTMLTextAreaElement>("status-message");

// GroupBox1 "unixtimeを普通にする": mirrors textBoxFromUnixtime_Leave.
function bindUnixtimeGroup() {
  const input = el<HTMLInputElement>("input-unixtime");
  const output = el<HTMLInputElement>("output-localdatetime");

  input.addEventListener("blur", async () => {
    const result = await invoke<string | null>("convert_unixtime_input", {
      input: input.value,
    });
    output.value = result ?? "";
  });
}

// GroupBox2 "日付け文字列をunixtime": mirrors textBox4_Leave and
// textBoxISO8601Example_Enter.
function bindIso8601Group() {
  const input = el<HTMLInputElement>("input-iso8601");
  const output = el<HTMLInputElement>("output-unixtime");
  const example = el<HTMLInputElement>("example-iso8601");

  input.addEventListener("blur", async () => {
    if (input.value.trim() === "") return;
    const result = await invoke<ConversionResult>("convert_iso8601_input", {
      input: input.value,
    });
    output.value = result.unixtime ?? "";
    statusMessage().value = result.message;
  });

  example.addEventListener("focus", async () => {
    const message = await invoke<string>("copy_iso8601_example", {
      text: example.value,
    });
    statusMessage().value = message;
    example.select();
  });
}

// Top "Action" menu, mirroring menuStrip1 / contextMenuStrip1.
function bindActionMenu() {
  const menuButton = el<HTMLButtonElement>("menu-action-btn");
  const dropdown = el<HTMLUListElement>("menu-action-dropdown");

  const closeMenu = () => dropdown.classList.add("hidden");

  menuButton.addEventListener("click", (e) => {
    e.stopPropagation();
    dropdown.classList.toggle("hidden");
  });
  document.addEventListener("click", closeMenu);

  const commandByAction: Record<string, string> = {
    copy_unixtime: "action_copy_unixtime",
    copy_ymd1: "action_copy_ymd1",
    copy_ymd2: "action_copy_ymd2",
    exit: "action_exit",
    announce_p: "action_announce_p",
  };

  dropdown.querySelectorAll<HTMLLIElement>("li[data-action]").forEach((item) => {
    item.addEventListener("click", async () => {
      closeMenu();
      const action = item.dataset.action;
      if (!action) return;
      const command = commandByAction[action];
      const result = await invoke<string>(command);
      if (action !== "exit") {
        statusMessage().value = result;
      }
    });
  });
}

// Mirrors Form1_Load's initial ISO8601 sample value.
async function loadIso8601Example() {
  const example = el<HTMLInputElement>("example-iso8601");
  example.value = await invoke<string>("get_iso8601_example_now");
}

window.addEventListener("DOMContentLoaded", () => {
  bindUnixtimeGroup();
  bindIso8601Group();
  bindActionMenu();
  void loadIso8601Example();
});
