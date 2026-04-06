onmessage = e => {
    e.waitUntil(import("./file", { with: { type: "text" } })
        .then(module => e.source.postMessage("LOADED"))
        .catch(error => e.source.postMessage("FAILED")));
  };
