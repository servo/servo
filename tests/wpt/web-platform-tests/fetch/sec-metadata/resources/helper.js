function assert_header_equals(value, expected) {
  if (typeof(value) === "string"){
    assert_not_equals(value, "No header has been recorded");
    value = JSON.parse(value);
  }
  assert_equals(value.dest, expected.dest, "dest");
  // Mode is commented out as no test cases have been filled out yet
  // assert_equals(value.mode, expected.mode, "mode");
  assert_equals(value.site, expected.site, "site");
  assert_equals(value.user, expected.user, "user");
}
