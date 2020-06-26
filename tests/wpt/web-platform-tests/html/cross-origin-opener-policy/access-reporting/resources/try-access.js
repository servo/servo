// A function trying to access to |w| through a "CrossOrigin" attribute (blur).
// This function is kept in its own file to ensure the source location of the
// call stays constant.
function tryAccess(w) {
  try {
    w.blur();
  } catch(e) {}
}

function assert_source_location_found(report) {
  assert_true(report.body["source-file"].includes("try-access.js"));
  assert_equals(report.body["lineno"], 6);
  assert_equals(report.body["colno"], 7);
}

function assert_source_location_missing(report) {
  assert_equals(report.body["source-file"], undefined);
  assert_equals(report.body["lineno"], undefined);
  assert_equals(report.body["colno"], undefined);
}
