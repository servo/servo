// META: timeout=long
// META: script=/common/get-host-info.sub.js
// META: script=./resources/common.js
const {ORIGIN, REMOTE_ORIGIN} = get_host_info();
const COEP = '|header(cross-origin-embedder-policy,credentialless)';
const COEP_RO =
  '|header(cross-origin-embedder-policy-report-only,credentialless)';
const CORP_CROSS_ORIGIN =
  '|header(cross-origin-resource-policy,cross-origin)';
const FRAME_URL = `${ORIGIN}/common/blank.html?pipe=`;
const REMOTE_FRAME_URL = `${REMOTE_ORIGIN}/common/blank.html?pipe=`;

function checkCorpReport(report, contextUrl, blockedUrl, disposition) {
  assert_equals(report.type, 'coep');
  assert_equals(report.url, contextUrl);
  assert_equals(report.body.type, 'corp');
  assert_equals(report.body.blockedURL, blockedUrl);
  assert_equals(report.body.disposition, disposition);
  assert_equals(report.body.destination, 'iframe');
}

function checkCoepMismatchReport(report, contextUrl, blockedUrl, disposition) {
  assert_equals(report.type, 'coep');
  assert_equals(report.url, contextUrl);
  assert_equals(report.body.type, 'navigation');
  assert_equals(report.body.blockedURL, blockedUrl);
  assert_equals(report.body.disposition, disposition);
}

function loadFrame(document, url) {
  return new Promise((resolve, reject) => {
    const frame = document.createElement('iframe');
    frame.src = url;
    frame.onload = () => resolve(frame);
    frame.onerror = reject;
    document.body.appendChild(frame);
  });
}

// |parentSuffix| is a suffix for the parent frame URL.
// |targetUrl| is a URL for the target frame.
async function loadFrames(test, parentSuffix, targetUrl) {
  const frame = await loadFrame(document, FRAME_URL + parentSuffix);
  test.add_cleanup(() => frame.remove());
  // Here we don't need "await". This loading may or may not succeed, and
  // we're not interested in the result.
  loadFrame(frame.contentDocument, targetUrl);

  return frame;
}

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

  // Wait 500ms more to catch additionnal unexpected reports.
  await receivedEveryReports;
  await new Promise(r => step_timeout(r, 500));
  return reports;
}

function desc(headers) {
  return headers === '' ? '(none)' : headers;
}

// CASES is a list of test case. Each test case consists of:
//   parent_headers: the suffix of the URL of the parent frame.
//   target_headers: the suffix of the URL of the target frame.
//   expected_reports: one of:
//     'CORP':    CORP violation
//     'CORP-RO': CORP violation (report only)
//     'NAV':     COEP mismatch between the frames.
//     'NAV-RO':  COEP mismatch between the frames (report only).
const reportingTest = function(
  parent_headers, target_headers, expected_reports) {
  // These tests are very slow, so they must be run in parallel using
  // async_test.
  promise_test_parallel(async t => {
    const targetUrl = REMOTE_FRAME_URL + target_headers;
    const parent = await loadFrames(t, parent_headers, targetUrl);
    const contextUrl = parent.src ? parent.src : 'about:blank';
    const reports = await observeReports(
        parent.contentWindow,
        expected_reports.length
      );
    assert_equals(reports.length, expected_reports.length);
    for (let i = 0; i < reports.length; i += 1) {
      const report = reports[i];
      switch (expected_reports[i]) {
        case 'CORP':
          checkCorpReport(report, contextUrl, targetUrl, 'enforce');
          break;
        case 'CORP-RO':
          checkCorpReport(report, contextUrl, targetUrl, 'reporting');
          break;
        case 'NAV':
          checkCoepMismatchReport(report, contextUrl, targetUrl, 'enforce');
          break;
        case 'NAV-RO':
          checkCoepMismatchReport(report, contextUrl, targetUrl, 'reporting');
          break;
        default:
          assert_unreached(
            'Unexpected report exception: ' + expected_reports[i]);
      }
    }
  }, `parent: ${desc(parent_headers)}, target: ${desc(target_headers)}, `);
}

reportingTest('', '', []);
reportingTest('', COEP, []);
reportingTest(COEP, COEP, ['CORP']);
reportingTest(COEP, '', ['CORP']);

reportingTest('', CORP_CROSS_ORIGIN, []);
reportingTest(COEP, CORP_CROSS_ORIGIN, ['NAV']);

reportingTest('', COEP + CORP_CROSS_ORIGIN, []);
reportingTest(COEP, COEP + CORP_CROSS_ORIGIN, []);

reportingTest(COEP_RO, COEP, ['CORP-RO']);
reportingTest(COEP_RO, '', ['CORP-RO', 'NAV-RO']);
reportingTest(COEP_RO, CORP_CROSS_ORIGIN, ['NAV-RO']);
reportingTest(COEP_RO, COEP + CORP_CROSS_ORIGIN, []);

reportingTest(COEP, COEP_RO + CORP_CROSS_ORIGIN, ['NAV']);
