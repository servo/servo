// META: script=/common/get-host-info.sub.js
// META: script=/reporting/resources/report-helper.js
// META: script=/service-workers/service-worker/resources/test-helpers.sub.js
//
// The following tests assume these headers have been set for the document.
// Reporting-Endpoints:
// endpoint="/reporting/resources/report.py?reportID=002ff910-653f-4ba7-ae2f-e62f74b7d0ce"
// Connection-Allowlist: (response-origin
// "https://{{hosts[alt][]}}:{{ports[https][0]}}"); report-to=endpoint
// Connection-Allowlist-Report-Only: (response-origin
// "https://{{hosts[alt][www]}}:{{ports[https][0]}}"); report-to=endpoint
//
// The service worker script does not set an allowlist, meaning that any request
// from the service worker should be allowed. However, the service worker should
// still respect the *document's* allowlists, meaning requests originating from
// the document that don't match the above headers should still be acted upon.

const port = get_host_info().HTTPS_PORT_ELIDED;
const SUCCESS = true;
const FAILURE = false;

async function service_worker_reporting_test(t, origin, expectation) {
  // We need the service worker's scope here to be global, because we want this
  // document with its own Connection Allowlist to be under worker control. By
  // default the maximum scope of a service worker is the directory where its
  // script lives. We override this via the Service-Worker-Allowed header on the
  // worker script resource.
  const scope = '/';
  const registration = await service_worker_unregister_and_register(
      t, 'resources/service-worker-fetch-script-allow-all.js', scope);
  t.add_cleanup(async () => {
    await registration.unregister();
  })
  const worker = get_newest_worker(registration);
  await wait_for_state(t, worker, 'activated');

  // Wait for document to be controlled by the service worker.
  await new Promise((resolve) => {
    if (navigator.serviceWorker.controller) {
      return resolve();
    }
    navigator.serviceWorker.addEventListener('controllerchange',
                                             () => resolve());
  });

  const fetch_url = `${origin}/common/blank-with-cors.html`;
  let result;
  try {
    const response =
        await fetch(fetch_url, {mode: 'cors', credentials: 'omit'});
    result = response.ok;
  } catch (err) {
    result = false;
  }

  if (expectation === SUCCESS) {
    assert_true(result, `Fetch to ${origin} should succeed.`);
  } else {
    assert_false(result, `Fetch to ${origin} should be blocked.`);
  }
}

// 1. Fetch to {{hosts[alt][]}} should succeed because it matches the
// enforcement allowlist, but should send a report-disposition report because it
// doesn't match the report-only allowlist.
promise_test(async t => {
  await service_worker_reporting_test(t, 'https://{{hosts[alt][]}}' + port,
                                      SUCCESS);
}, 'Report sent in report-only mode');

// 2. Fetch to {{hosts[alt][www]}} should fail because it doesn't match the
// enforcement allowlist. It should only generate an enforce-disposition report,
// because it *does* match the report-only allowlist.
promise_test(async t => {
  await service_worker_reporting_test(
      t,
      'https://{{hosts[alt][www]}}' + port,
      FAILURE,
  );
}, 'Report sent in enforcement mode.');

// The above tests should have generated two reports: one in enforce disposition
// and another in report disposition.
promise_test(async t => {
  const endpoint = '/reporting/resources/report.py';
  const id = '002ff910-653f-4ba7-ae2f-e62f74b7d0ce';
  await wait(5000);
  let reports = await pollReports(endpoint, id);
  assert_equals(reports.length, 2);

  // Sort reports by disposition.
  reports = reports.sort((r1, r2) => {
    if (r1['body']['disposition'] < r2['body']['disposition']) {
      return -1;
    }
    if (r1['body']['disposition'] > r2['body']['disposition']) {
      return 1;
    }
    return 0;
  });

  // The first report was generated because it violated the enforced allowlist.
  assert_equals(reports[0]['type'], 'connection-allowlist');
  assert_equals(reports[0]['url'], location.href);
  assert_equals(reports[0]['body']['disposition'], 'enforce');
  assert_equals(
      reports[0]['body']['connection'],
      `https://{{hosts[alt][www]}}${port}/common/blank-with-cors.html`);

  // The second report was generated because it violated the report-only
  // allowlist.
  assert_equals(reports[1]['type'], 'connection-allowlist');
  assert_equals(reports[1]['url'], location.href);
  assert_equals(reports[1]['body']['disposition'], 'report');
  assert_equals(reports[1]['body']['connection'],
                `https://{{hosts[alt][]}}${port}/common/blank-with-cors.html`);
});
