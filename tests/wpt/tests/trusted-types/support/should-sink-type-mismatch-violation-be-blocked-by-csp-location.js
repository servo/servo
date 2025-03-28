function passPlainStringToTrustedTypeSink() {       // 1
  return trusted_type_violation_for(TypeError, _ => // 2
    setTimeout    (";;;;;;;;;;;;;;;;;;;;;;;;;;;;;") //_3
/*  |
12345678901234567890
*/
  );
}

promise_test(async () => {
  let violation = await passPlainStringToTrustedTypeSink();
  let baseURL = (new URL(location.href)).origin;
  let sourceFile = new URL("/trusted-types/support/should-sink-type-mismatch-violation-be-blocked-by-csp-location.js", baseURL).toString();
  assert_equals(violation.sourceFile, sourceFile, "source file");
  assert_equals(violation.lineNumber, 3, "line number");
  // https://w3c.github.io/webappsec-csp/#create-violation-for-global does not
  // say how to determine the location and browsers provide inconsistent values
  // for column number, so just check it's at least the offset of the 's'
  // character of setTimeout.
  assert_greater_than_equal(violation.columnNumber, 5, "column number");
} , `Location of required-trusted-types-for violations.`);
