// META: global=window,worker

// Report, ReportBody, and the dictionaries that inherit from ReportBody.
for (const name of [
  "Report",
  "ReportBody",
  "ConnectionAllowlistViolationReport",
  "CrashReportBody",
  "CSPViolationReportBody",
  "DeprecationReportBody",
  "IntegrityViolationReportBody",
  "InterventionReportBody",
  "PermissionsPolicyViolationReportBody",
  "ScriptingPolicyReportBody",
]) {
  test(() => {
    assert_false(name in self);
  }, `${name} is not exposed`);
}
