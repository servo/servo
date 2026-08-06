// META: script=/common/get-host-info.sub.js
// META: script=/reporting/resources/report-helper.js
// META: script=/service-workers/service-worker/resources/test-helpers.sub.js

const port = get_host_info().HTTPS_PORT_ELIDED;
const reportID = 'a1b2c3d4-5678-90ef-1234-567890abcdef';
const endpoint = '/reporting/resources/report.py';

promise_test(async t => {
  const scope = '/';
  const script = 'resources/service-worker-fetch-script-allow-all.js';

  const registration = await service_worker_unregister_and_register(t, script, scope);
  t.add_cleanup(async () => {
    await registration.unregister();
  });

  const worker = get_newest_worker(registration);
  await wait_for_state(t, worker, 'activated');

  await new Promise((resolve) => {
    if (navigator.serviceWorker.controller) {
      return resolve();
    }
    navigator.serviceWorker.addEventListener('controllerchange', () => resolve());
  });

  const unstrippedUrl = `https://{{hosts[alt][www]}}${port}/common/blank-with-cors.html?param=val#test-fragment`;
  const expectedSanitizedUrl = `https://{{hosts[alt][www]}}${port}/common/blank-with-cors.html?param=val`;

  let fetchFailed = false;
  try {
    await fetch(unstrippedUrl, { mode: 'cors', credentials: 'omit' });
  } catch (err) {
    fetchFailed = true;
  }
  assert_true(fetchFailed, 'Fetch violating Connection-Allowlist should be blocked.');

  await wait(5000);
  const reports = await pollReports(endpoint, reportID);

  const violationReport = reports.find(r => r.type === 'connection-allowlist');
  assert_not_equals(violationReport, undefined, 'Connection-Allowlist violation report should be generated.');
  assert_equals(violationReport.body.connection, expectedSanitizedUrl, 'Fragment (#hash) must be stripped from report.body.connection.');
  assert_false(violationReport.body.connection.includes('#test-fragment'), 'Report connection URL must not contain the fragment.');

}, 'Service Worker intercepted fetch reports sanitize target connection URLs.');
