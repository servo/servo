import("./basic.css")
  .then(() => postMessage("LOADED"))
  .catch(e => postMessage("NOT LOADED"));