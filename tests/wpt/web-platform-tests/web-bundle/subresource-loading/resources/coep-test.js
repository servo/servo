async function expectCOEPReport(func) {
  const reportsPromise = new Promise((resolve) => {
    const observer = new ReportingObserver((reports) => {
      observer.disconnect();
      resolve(reports.map(r => r.toJSON()));
    });
    observer.observe();
  });

  await func();

  const reports = await reportsPromise;
  assert_equals(reports.length, 1);
  assert_equals(reports[0].type, "coep");
  assert_equals(reports[0].url, location.href);
  return reports[0];
}

const prefix = 'https://www1.web-platform.test:8444/web-bundle/resources/wbn/cors/';
const no_corp_url = 'urn:uuid:5eafff38-e0a0-4661-bde0-434255aa9d93';
const corp_same_origin_url = 'urn:uuid:7e13b47a-8b91-4a0e-997c-993a5e2f3a34';
const corp_cross_origin_url = 'urn:uuid:86d5b696-8867-4454-8b07-51239a0817f7';

promise_test(async () => {
  const report = await expectCOEPReport(async () => {
    await addScriptAndWaitForError(prefix + 'no-corp.js');
  });
  assert_equals(report.body.blockedURL, prefix + 'no-corp.js');
  assert_equals(report.body.type, "corp");
  assert_equals(report.body.disposition, "enforce");
  assert_equals(report.body.destination, "script");
}, "Cross-origin subresource without Cross-Origin-Resource-Policy: header should be blocked and generate a report.");

promise_test(async () => {
  await addScriptAndWaitForError(prefix + 'corp-same-origin.js');
}, "Cross-origin subresource with Cross-Origin-Resource-Policy: same-origin should be blocked.");

promise_test(async () => {
  await addScriptAndWaitForExecution(prefix + 'corp-cross-origin.js');
}, "Cross-origin subresource with Cross-Origin-Resource-Policy: cross-origin should be loaded.");

promise_test(async () => {
  const report = await expectCOEPReport(async () => {
    const iframe = document.createElement('iframe');
    iframe.src = no_corp_url;
    document.body.appendChild(iframe);
  });

  assert_equals(report.body.blockedURL, no_corp_url);
  assert_equals(report.body.type, "corp");
  assert_equals(report.body.disposition, "enforce");
  assert_equals(report.body.destination, "iframe");
}, "Urn:uuid iframe without Cross-Origin-Resource-Policy: header should be blocked and generate a report.");

promise_test(async () => {
  const report = await expectCOEPReport(async () => {
    const iframe = document.createElement('iframe');
    iframe.src = corp_same_origin_url;
    document.body.appendChild(iframe);
  });

  assert_equals(report.body.blockedURL, corp_same_origin_url);
  assert_equals(report.body.type, "corp");
  assert_equals(report.body.disposition, "enforce");
  assert_equals(report.body.destination, "iframe");
}, "Urn:uuid iframe with Cross-Origin-Resource-Policy: same-origin should be blocked and generate a report.");

promise_test(async () => {
  const iframe = document.createElement('iframe');
  iframe.src = corp_cross_origin_url;
  await addElementAndWaitForLoad(iframe);
  assert_equals(
    await evalInIframe(iframe, 'location.href'),
    corp_cross_origin_url);
}, "Urn:uuid iframe with Cross-Origin-Resource-Policy: cross-origin should not be blocked.");
