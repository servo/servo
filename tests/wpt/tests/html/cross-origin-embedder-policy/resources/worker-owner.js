const is_worker = !('window' in self);
const parent_or_self = is_worker ? self : self.parent;

function startWorkerAndObserveReports(worker_url, wait_for_report) {
  const worker = new Worker(worker_url);
  const result_promise = new Promise(resolve => {
    worker.onmessage = _ => resolve('success');
    worker.onerror = _ => resolve('error');
  });
  worker.postMessage("postMessage('reply to owner from worker');");

  const report_promise = new Promise(resolve => {
    const observer = new ReportingObserver(reports => {
      observer.disconnect();
      resolve(reports.map(r => r.toJSON()));
    });
    observer.observe();
  });

  if (wait_for_report) {
    Promise.all([result_promise, report_promise]).then(results => {
      parent_or_self.postMessage(results[1]);
    });
  } else {
    result_promise.then(result => {
      parent_or_self.postMessage([]);
    });
  }
}

if (is_worker) {
  onmessage = e => {
    startWorkerAndObserveReports(e.data.worker_url, e.data.wait_for_report);
  };
}

