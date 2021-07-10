function run({script, port}) {
  const reports = [];
  const observer = new ReportingObserver((rs) => {
    for (const r of rs) {
      reports.push(r.toJSON());
    }
  });
  // Wait 200ms for reports to settle.
  setTimeout(() => {
    observer.disconnect();
    port.postMessage(reports);
  }, 200);
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
