function wrap_by_tag(tag, text) {
  return tag ? `${tag}: ${text}`: text;
}

function assert_header_equals(value, expected, tag) {
  if (typeof(value) === "string"){
    assert_not_equals(value, "No header has been recorded");
    value = JSON.parse(value);
  }

  assert_equals(value.dest, expected.dest, wrap_by_tag(tag, "dest"));
  assert_equals(value.mode, expected.mode, wrap_by_tag(tag, "mode"));
  assert_equals(value.site, expected.site, wrap_by_tag(tag, "site"));
  if (expected.hasOwnProperty("user"))
    assert_equals(value.user, expected.user, wrap_by_tag(tag, "user"));
}
