function assert_header_equals(value, expected) {
  if (typeof(value) === "string"){
    assert_not_equals(value, "No header has been recorded");
    value = JSON.parse(value);
  }
  assert_equals(value.dest, expected.dest, "dest");
  assert_equals(value.mode, expected.mode, "mode");
  assert_equals(value.site, expected.site, "site");
  if (expected.hasOwnProperty("user"))
    assert_equals(value.user, expected.user, "user");
}
