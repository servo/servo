onmessage = e => {
    e.waitUntil(import("./module.json", { with: { type: "json" } })
        .then(module => e.source.postMessage("LOADED"))
        .catch(error => e.source.postMessage("FAILED")));
  };