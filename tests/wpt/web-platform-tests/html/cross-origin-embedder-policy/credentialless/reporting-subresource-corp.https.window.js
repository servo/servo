// META: timeout=long
// META: script=/common/utils.js
// META: script=/common/get-host-info.sub.js
// META: script=/service-workers/service-worker/resources/test-helpers.sub.js
const {ORIGIN, REMOTE_ORIGIN} = get_host_info();
const BASE = "/html/cross-origin-embedder-policy/resources";
const REPORTING_FRAME_URL = `${ORIGIN}${BASE}/reporting-empty-frame.html` +
  '?pipe=header(cross-origin-embedder-policy,credentialless)' +
  '&token=${token()}';

async function observeReports(global, expected_count) {
  const reports = [];
  const receivedEveryReports = new Promise(resolve => {
    if (expected_count == 0)
      resolve();

    const observer = new global.ReportingObserver((rs) => {
      for (const r of rs) {
        reports.push(r.toJSON());
      }
      if (expected_count <= reports.length)
        resolve();
    });
    observer.observe();

  });

  await receivedEveryReports;
  // Wait 1000ms more to catch additionnal unexpected reports.
  await new Promise(r => step_timeout(r, 1000));
  return reports;
}

async function fetchInFrame(t, frameUrl, url, expected_count) {
  const frame = await with_iframe(frameUrl);
  t.add_cleanup(() => frame.remove());

  const init = { mode: 'no-cors', cache: 'no-store' };
  let future_reports = observeReports(frame.contentWindow, expected_count);
  await frame.contentWindow.fetch(url, init).catch(() => {});

  return await future_reports;
}

function checkReport(report, contextUrl, blockedUrl, disposition, destination) {
  assert_equals(report.type, 'coep');
  assert_equals(report.url, contextUrl);
  assert_equals(report.body.type, 'corp');
  assert_equals(report.body.blockedURL, blockedUrl);
  assert_equals(report.body.disposition, disposition);
  assert_equals(report.body.destination, destination);
}

// A redirection is used, so that the initial request is same-origin and is
// proxyied through the service worker. The ServiceWorker is COEP:unsafe-none,
// so it will make the cross-origin request with credentials. The fetch will
// succeed, but the response will be blocked by CORP when entering the
// COEP:credentialless document.
// https://github.com/w3c/ServiceWorker/issues/1592
promise_test(async (t) => {
  const url = `${ORIGIN}/common/redirect.py?location=` +
       encodeURIComponent(`${REMOTE_ORIGIN}/common/text-plain.txt`);
  const WORKER_URL = `${ORIGIN}${BASE}/sw.js`;
  const reg = await service_worker_unregister_and_register(
    t, WORKER_URL, REPORTING_FRAME_URL);
  t.add_cleanup(() => reg.unregister());
  const worker = reg.installing || reg.waiting || reg.active;
  worker.addEventListener('error', t.unreached_func('Worker.onerror'));
  await wait_for_state(t, worker, 'activated');

  const reports = await fetchInFrame(t, REPORTING_FRAME_URL, url, 1);
  assert_equals(reports.length, 1);
  checkReport(reports[0], REPORTING_FRAME_URL, url, 'enforce', '');
});
