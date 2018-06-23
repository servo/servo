function parse_metadata(value) {
  let result = {};
  value.split(',').forEach(item => {
    let parsed = item.trim().split('=');
    parsed[1] = parsed[1].trim().replace(/^"|"$/g, '');
    result[parsed[0]] = parsed[1];
  });
  return result;
}

function assert_header_equals(value, expected) {
  let result = parse_metadata(value);
  assert_equals(result.cause, expected.cause, "cause");
  assert_equals(result.destination, expected.destination, "destination");
  assert_equals(result.target, expected.target, "target");
  assert_equals(result.site, expected.site, "site");
}
