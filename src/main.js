const { invoke } = window.__TAURI__.core;
const { listen } = window.__TAURI__.event;


/* listen("my_event", (eventPayload) => {
  console.log(eventPayload.payload)
  output.innerHTML = eventPayload.payload
}) */


async function init(){
  await invoke("init");
}

async function perform_backup() {
  await invoke("perform_backup");
}

async function pick_fld(fld) {
  await invoke("pick_folders", {fld: fld});
}


//Der Button-clik leert das output fenster, bei my_event, wird etwas in de output appended

window.addEventListener("DOMContentLoaded", () => {
  let output = document.getElementById("output");
  let right_div = document.getElementById("right-div");

  listen("my_event", (eventPayload) => {
    console.log(eventPayload.payload);
    output.innerHTML += eventPayload.payload;
    right_div.scrollTo(0, output.scrollHeight);
    
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

  document.querySelector("#see-folders").addEventListener("click", (e) => {
    e.preventDefault();
    output.innerHTML = ""
    init();
  });

  init();


});
