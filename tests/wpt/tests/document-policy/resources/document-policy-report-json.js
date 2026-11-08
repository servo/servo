/**
 * @fileoverview functions for ensuring document policy report is serializable
 */

const check_report_json = (report) => {
  // Ensures JSON.stringify() serializes the report correctly.
  const report_json = JSON.parse(JSON.stringify(report));
  assert_equals(report.type, report_json.type);
  assert_equals(report.url, report_json.url);
  assert_equals(report.body.featureId, report_json.body.featureId);
  assert_equals(report.body.disposition, report_json.body.disposition);
  assert_equals(report.body.sourceFile, report_json.body.sourceFile);
  assert_equals(report.body.lineNumber, report_json.body.lineNumber);
  assert_equals(report.body.columnNumber, report_json.body.columnNumber);
}