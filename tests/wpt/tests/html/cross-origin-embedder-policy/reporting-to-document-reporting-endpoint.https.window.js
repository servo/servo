// META: script=/common/get-host-info.sub.js
// META: script=/common/utils.js

// This file consists of tests for COEP reporting using Reporting-Endpoints
// header. It exclusively tests that reports can be sent to Reporting-Endpoint
// configured endpoint.
const { REMOTE_ORIGIN } = get_host_info();

const REPORT_ENDPOINT = token();
const REPORT_ONLY_ENDPOINT = token();
const FRAME_URL = `resources/reporting-empty-frame.html` +
  `?pipe=header(cross-origin-embedder-policy,require-corp;report-to="endpoint")` +
  `|header(cross-origin-embedder-policy-report-only,require-corp;report-to="report-only-endpoint")` +
  `|header(reporting-endpoints, endpoint="/html/cross-origin-embedder-policy/resources/report.py?key=${REPORT_ENDPOINT}"\\, report-only-endpoint="/html/cross-origin-embedder-policy/resources/report.py?key=${REPORT_ONLY_ENDPOINT}")`;

function wait(ms) {
  return new Promise(resolve => step_timeout(resolve, ms));
}

async function fetchReports(endpoint) {
  const res = await fetch(`resources/report.py?key=${endpoint}`, {
    cache: 'no-store'
  });
  if (res.status == 200) {
    return await res.json();
  }
  return [];
}

async function fetchCoepReport(
  endpoint, type, blockedUrl, contextUrl, disposition) {
  blockedUrl = new URL(blockedUrl, location).href;
  contextUrl = new URL(contextUrl, location).href;
  const reports = await fetchReports(endpoint);
  return reports.find(r => (
    r.type == 'coep' &&
    r.url == contextUrl &&
    r.body.type == type &&
    r.body.blockedURL === blockedUrl &&
    r.body.disposition === disposition));
}

async function checkCorpReportExists(
  endpoint, blockedUrl, contextUrl, destination, disposition) {
  blockedUrl = new URL(blockedUrl, location).href;
  contextUrl = new URL(contextUrl, location).href;
  contextUrl.replace(REPORT_ENDPOINT, "REPORT_ENDPOINT_UUID");
  contextUrl.replace(REPORT_ONLY_ENDPOINT, "REPORT_ONLY_ENDPOINT_UUID");
  const report = await fetchCoepReport(
    endpoint, 'corp', blockedUrl, contextUrl, disposition);
  assert_true(!!report,
    `A corp report with blockedURL ${blockedUrl.split("?")[0]} ` +
    `and url ${contextUrl} is not found.`);
  assert_equals(report.body.destination, destination);
}

async function checkNavigationReportExists(
  endpoint, blockedUrl, contextUrl, disposition) {
  blockedUrl = new URL(blockedUrl, location).href;
  contextUrl = new URL(contextUrl, location).href;
  contextUrl.replace(REPORT_ENDPOINT, "REPORT_ENDPOINT_UUID");
  contextUrl.replace(REPORT_ONLY_ENDPOINT, "REPORT_ONLY_ENDPOINT_UUID");
  const report = await fetchCoepReport(
    endpoint, 'navigation', blockedUrl, contextUrl, disposition);
  assert_true(!!report,
    `A navigation report with blockedURL ${blockedUrl.split("?")[0]} ` +
    `and url ${contextUrl} is not found.`);
}

promise_test(async t => {
  const iframe = document.createElement('iframe');
  t.add_cleanup(() => iframe.remove());

  iframe.src = FRAME_URL;
  await new Promise(resolve => {
    iframe.addEventListener('load', resolve, { once: true });
    document.body.appendChild(iframe);
  });

  const url = `${REMOTE_ORIGIN}/common/text-plain.txt?${token()}`;
  const init = { mode: 'no-cors', cache: 'no-store' };
  // The response comes from cross-origin, and doesn't have a CORP
  // header, so it is blocked.
  iframe.contentWindow.fetch(url, init).catch(() => { });

  // Wait for reports to be uploaded.
  await wait(1000);
  await checkCorpReportExists(
    REPORT_ENDPOINT, url, iframe.src, '', 'enforce');
  await checkCorpReportExists(
    REPORT_ONLY_ENDPOINT, url, iframe.src, '', 'reporting');
}, 'subresource CORP');

promise_test(async t => {
  const iframe = document.createElement('iframe');
  t.add_cleanup(() => iframe.remove());

  iframe.src = FRAME_URL;
  await new Promise(resolve => {
    iframe.addEventListener('load', resolve, { once: true });
    document.body.appendChild(iframe);
  });

  const url = `${REMOTE_ORIGIN}/common/blank.html?${token()}`;
  // The nested frame comes from cross-origin and doesn't have a CORP
  // header, so it is blocked.
  const nested = iframe.contentWindow.document.createElement('iframe');
  nested.src = url;
  iframe.contentWindow.document.body.appendChild(nested);

  // Wait for reports to be uploaded.
  await wait(1000);
  await checkCorpReportExists(
    REPORT_ENDPOINT, url, iframe.src, 'iframe', 'enforce');
  await checkCorpReportExists(
    REPORT_ONLY_ENDPOINT, url, iframe.src, 'iframe', 'reporting');
}, 'navigation CORP on cross origin');

promise_test(async (t) => {
  const iframe = document.createElement('iframe');
  t.add_cleanup(() => iframe.remove());

  iframe.src = FRAME_URL;
  const targetUrl = `/common/blank.html?${token()}`;
  iframe.addEventListener('load', t.step_func(() => {
    const nested = iframe.contentDocument.createElement('iframe');
    nested.src = targetUrl;
    // |nested| doesn't have COEP whereas |iframe| has, so it is blocked.
    iframe.contentDocument.body.appendChild(nested);
  }), { once: true });

  document.body.appendChild(iframe);

  // Wait for reports to be uploaded.
  await wait(1000);
  await checkNavigationReportExists(
    REPORT_ENDPOINT, targetUrl, iframe.src, 'enforce');
  await checkNavigationReportExists(
    REPORT_ONLY_ENDPOINT, targetUrl, iframe.src, 'reporting');
}, 'navigation CORP on same origin');
