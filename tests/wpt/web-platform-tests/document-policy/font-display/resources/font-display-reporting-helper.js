function check_report_format(report, expected_url, expected_disposition) {
  assert_equals(report.type, 'document-policy-violation');
  assert_equals(report.url, expected_url);
  assert_equals(report.body.featureId, 'font-display-late-swap');
  assert_equals(report.body.disposition, expected_disposition);
  assert_true('sourceFile' in report.body);
  assert_true('lineNumber' in report.body);
  assert_true('columnNumber' in report.body);
}

function check_violation_report_format(report, expected_url) {
  check_report_format(report, expected_url, 'enforce');
}

function check_report_only_report_format(report, expected_url) {
  check_report_format(report, expected_url, 'report');
}

function makeFontFaceDeclaration(family, display) {
  url = '/fonts/Ahem.ttf?pipe=trickle(d1)'; // Before the swap period is over
  return `@font-face { font-family: ${family}; src: url("${url}"); font-display: ${display}; }`;
}

/**
 * Run font-display test with given parameters.
 *
 * A violation report is expected with fontDisplayValue set to
 * ['', 'auto', 'block', 'swap']
 *
 * No violation report is expected with fontDisplayValue set to
 * ['fallback', 'optional']

 * @param {String} fontDisplayValue
 * @param {(Report, String) => () | undefined} format_check pass a callback to
 * check report format if a violation report is expected. If no report is
 * expected to be generated, leave this argument undefined.
 */
function runTest(fontDisplayValue, format_check) {
  window.onload = () => {
    const family = fontDisplayValue + '-face';
    const rule = makeFontFaceDeclaration(family, fontDisplayValue);

    const style = document.createElement('style');
    style.innerHTML = rule;
    document.body.appendChild(style);

    const div = document.createElement('div');
    div.textContent = 'a';
    div.style.fontFamily = family + ', Arial';
    document.body.appendChild(div);
  };

  const t = async_test('font-display-late-swap Report Format');

  new ReportingObserver(
    t.step_func_done((reports, _) => {
      assert_equals(reports.length, 1);
      assert_true(!!format_check);
      format_check(reports[0], document.location.href);
    }), {
      types: ['document-policy-violation'],
      buffered: true
    }
  ).observe();

  t.step_timeout(t.step_func_done(() => {
    assert_false(!!format_check, 'Expected violation report but did not get one.');
  }), 400); // 400ms should be sufficient to observe the violation report.
}

function testFontDisplayPolicyViolationGenerated(fontDisplayValue) {
  runTest(fontDisplayValue, check_violation_report_format);
}

function testFontDisplayPolicyReportOnlyGenerated(fontDisplayValue) {
  runTest(fontDisplayValue, check_report_only_report_format);
}

function testCompliantWithFontDisplayPolicy(fontDisplayValue) {
  runTest(fontDisplayValue);
}