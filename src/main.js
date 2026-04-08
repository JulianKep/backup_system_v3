const { invoke } = window.__TAURI__.core;
const { listen } = window.__TAURI__.event;

let output;

/* listen("my_event", (eventPayload) => {
  console.log(eventPayload.payload)
  output.innerHTML = eventPayload.payload
}) */

async function perform_backup() {
  await invoke("perform_backup");
}

async function pick_fld(fld) {
  await invoke("pick_folders", {fld: fld});
}


window.addEventListener("DOMContentLoaded", () => {
  output = document.getElementById("output");

  listen("my_event", (eventPayload) => {
    console.log(eventPayload.payload);
    output.innerHTML += eventPayload.payload;
  });

  document.querySelector("#backup-jetzt").addEventListener("click", (e) => {
    e.preventDefault();
    output.innerHTML = ""
    perform_backup();
  });

  document.querySelector("#pick-src").addEventListener("click", (e) => {
    e.preventDefault();
    output.innerHTML = ""
    pick_fld("src");
  });

  document.querySelector("#pick-dst").addEventListener("click", (e) => {
    e.preventDefault();
    output.innerHTML = ""
    pick_fld("dst");
  });


});

/* async function greet() {
  // Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
  greetMsgEl.textContent = await invoke("greet", { name: greetInputEl.value });
}
 */
