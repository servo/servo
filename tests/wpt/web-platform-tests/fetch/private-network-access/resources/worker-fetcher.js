const url = new URL(self.location).searchParams.get("url");
const worker = new Worker(url);

// Relay messages from the worker to the parent frame.
worker.addEventListener("message", (evt) => {
  self.postMessage(evt.data);
});

worker.addEventListener("error", (evt) => {
  self.postMessage({ error: evt.message || "unknown error" });
});
