import("./record-fetch.py?key={{GET[key]}}&action=incCount", { assert: { type: "css" } })
  .then(() => postMessage("LOADED"))
  .catch(e => postMessage("NOT LOADED"));