// A function trying to access to |w| through a "CrossOrigin" attribute (blur).
// This function is kept in its own file to ensure the source location of the
// call stays constant.
function tryAccess(w) {
  try {
    w.blur();
  } catch(e) {}
}

function assert_source_location_found(report) {
  assert_true(report.body.sourceFile.includes("try-access.js"));
  assert_equals(report.body.lineNumber, 6);
  assert_equals(report.body.columnNumber, 7);
}

function assert_source_location_missing(report) {
  assert_equals(report.body.sourceFile, undefined);
  assert_equals(report.body.lineNumber, undefined);
  assert_equals(report.body.columnNumber, undefined);
}
