import("./basic.css", { assert: { type: "css" } })
  .then(() => postMessage("LOADED"))
  .catch(e => postMessage("NOT LOADED"));