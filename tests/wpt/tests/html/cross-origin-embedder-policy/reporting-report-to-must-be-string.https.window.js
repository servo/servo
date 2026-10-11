// META: script=/common/get-host-info.sub.js
// META: timeout=long
// META: script=/common/utils.js

// The report-to parameter of Cross-Origin-Embedder-Policy and
// Cross-Origin-Embedder-Policy-Report-Only is only used when it is a structured
// field string. Each test pairs a non-string report-to on one header with a
// string report-to on the other, so the latter shows that reports are sent.
const { REMOTE_ORIGIN } = get_host_info();

function wait(ms) {
  return new Promise(resolve => step_timeout(resolve, ms));
}

async function fetchReports(key) {
  const res = await fetch(`resources/report.py?key=${key}`, {
    cache: 'no-store'
  });
  if (res.status == 200) {
    return await res.json();
  }
  return [];
}

async function fetchCorpReports(key, blockedURL) {
  const reports = await fetchReports(key);
  return reports.filter(r => (
    r.type == 'coep' &&
    r.body.type == 'corp' &&
    r.body.blockedURL === blockedURL));
}

// `reportTo` and `reportOnlyReportTo` are the report-to parameter values of the
// two headers. `expected` is the header whose endpoint receives the report.
function coepReportToTest(reportTo, reportOnlyReportTo, expected, description) {
  promise_test(async t => {
    const endpointKey = token();
    const reportOnlyEndpointKey = token();
    const reportURL = '/html/cross-origin-embedder-policy/resources/report.py';
    const frameURL = `resources/reporting-empty-frame.html` +
      `?pipe=header(cross-origin-embedder-policy,require-corp;report-to=${reportTo})` +
      `|header(cross-origin-embedder-policy-report-only,require-corp;report-to=${reportOnlyReportTo})` +
      `|header(reporting-endpoints, endpoint="${reportURL}?key=${endpointKey}"\\, report-only-endpoint="${reportURL}?key=${reportOnlyEndpointKey}")`;

    const iframe = document.createElement('iframe');
    t.add_cleanup(() => iframe.remove());
    iframe.src = frameURL;
    await new Promise(resolve => {
      iframe.addEventListener('load', resolve, { once: true });
      document.body.appendChild(iframe);
    });

    // The response comes from cross-origin and doesn't have a CORP header, so
    // it is blocked. This also shows that the policy itself is still enforced.
    const url = `${REMOTE_ORIGIN}/common/text-plain.txt?${token()}`;
    await promise_rejects_js(t, iframe.contentWindow.TypeError,
      iframe.contentWindow.fetch(url, { mode: 'no-cors', cache: 'no-store' }));

    // Wait for the report to the string report-to endpoint to be uploaded.
    // Reports from the same document are uploaded together, so once it
    // arrives, so would any report to the other endpoint.
    const [stringKey, otherKey] = expected === 'enforce' ?
      [endpointKey, reportOnlyEndpointKey] :
      [reportOnlyEndpointKey, endpointKey];
    let reports = [];
    while (reports.length === 0) {
      await wait(100);
      reports = await fetchCorpReports(stringKey, url);
    }
    assert_equals(reports.length, 1, 'reports for the string report-to');
    assert_equals(reports[0].body.disposition, expected);
    await wait(500);
    const otherReports = await fetchCorpReports(otherKey, url);
    assert_equals(otherReports.length, 0, 'reports for the other report-to');
  }, description);
}

coepReportToTest('endpoint', '"report-only-endpoint"', 'reporting',
  'Cross-Origin-Embedder-Policy ignores a token report-to');
coepReportToTest('"endpoint"', 'report-only-endpoint', 'enforce',
  'Cross-Origin-Embedder-Policy-Report-Only ignores a token report-to');
coepReportToTest(`:${btoa('endpoint')}:`, '"report-only-endpoint"', 'reporting',
  'Cross-Origin-Embedder-Policy ignores a byte sequence report-to');
coepReportToTest('"endpoint"', `:${btoa('report-only-endpoint')}:`, 'enforce',
  'Cross-Origin-Embedder-Policy-Report-Only ignores a byte sequence report-to');
