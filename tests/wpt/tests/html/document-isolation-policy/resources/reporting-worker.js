function run({script, port}) {
  const observer = new ReportingObserver((rs) => {
      for (const r of rs) {
        port.postMessage(r.toJSON());
      }
  });
  observer.observe();

  // This eval call may generate some reports.
  eval(script);
}

// For DedicatedWorker and ServiceWorker
self.addEventListener('message', (e) => run(e.data));

// For SharedWorker
self.addEventListener('connect', (e) => {
  e.ports[0].onmessage = (ev) => run(ev.data);
});
