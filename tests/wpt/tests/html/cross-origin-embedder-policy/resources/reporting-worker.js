async function run({script, port, expected_count = 0}) {
  const reports = [];
  let resolveEveryReport;
  const receivedEveryReport = new Promise(r => resolveEveryReport = r);
  const observer = new ReportingObserver((rs) => {
    reports.push(...rs);
    if (expected_count > 0 && reports.length >= expected_count) {
      resolveEveryReport();
    }
  });
  observer.observe();

  // This eval call may generate some reports.
  await eval(script);

  if (expected_count > 0) {
    // Wait until every expected report has been delivered, rather than for an
    // arbitrary amount of time.
    await receivedEveryReport;
  } else {
    // No reports are expected: collect anything already queued.
    reports.push(...observer.takeRecords());
  }
  observer.disconnect();
  port.postMessage(reports);
}

// For DedicatedWorker and ServiceWorker
self.addEventListener('message', (e) => run(e.data));

// For SharedWorker
self.addEventListener('connect', (e) => {
  e.ports[0].onmessage = (ev) => run(ev.data);
});
