// This is deliberately split from sync-script-reporting.html so that it will
// not be blocked by policy, and can actuall be executed.

var t = async_test("Sync-script Report Format");

var check_report_format = (reports, observer) => {
  let report = reports[0];
  assert_equals(report.type, "document-policy-violation");
  assert_equals(report.url, document.location.href);
  assert_equals(report.body.featureId, "sync-script");
  assert_equals(report.body.sourceFile, null);
  assert_equals(report.body.lineNumber, null);
  assert_equals(report.body.columnNumber, null);
  assert_equals(report.body.disposition, "enforce");
  check_report_json(report);
};

new ReportingObserver(t.step_func_done(check_report_format),
                      { types: ['document-policy-violation'],
                        buffered: true}).observe();
