function createForbiddenPolicy() {                  // 1
  return trusted_type_violation_for(TypeError, _ => // 2
    trustedTypes.                                   // 3
      createPolicy      ("tt-policy-name")          //_4
/*    |
1234567890123456789012345
*/
  );
}

promise_test(async () => {
  let violation = await createForbiddenPolicy();
  let baseURL = (new URL(location.href)).origin;
  let sourceFile = new URL("/trusted-types/support/should-trusted-type-policy-creation-be-blocked-by-csp-location.js", baseURL).toString();
  assert_equals(violation.sourceFile, sourceFile, "source file");
  assert_equals(violation.lineNumber, 4, "line number");
  // https://w3c.github.io/webappsec-csp/#create-violation-for-global does not
  // say how to determine the location and browsers provide inconsistent values
  // for column number, so just check it's at least the offset of the 'c'
  // character of createPolicy.
  assert_greater_than_equal(violation.columnNumber, 7, "column number");
} , `Location of trusted-types violations.`);
