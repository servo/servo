// META: title=`Origin` Construction and Parsing

//
// Opaque origins:
//
test(t => {
  const origin = new Origin();
  assert_true(origin.opaque, "Origin should be opaque.");
  assert_equals(origin.toJSON(), "null", "toJSON() should return 'null'.");
}, "Default-constructed Origin is opaque.");

test(t => {
  const origin = new Origin("null");
  assert_true(origin.opaque, "Origin should be opaque.");
  assert_equals(origin.toJSON(), "null", "toJSON() should return 'null'.");
}, "Origin constructed with 'null' is opaque.");

test(t => {
  const origin = Origin.parse("null");
  assert_true(origin.opaque, "Origin should be opaque.");
  assert_equals(origin.toJSON(), "null", "toJSON() should return 'null'.");
}, "Origin parsed from 'null' is opaque.");

//
// Invalid serializations:
//
const invalidSerializations = [
  "",
  "invalid",
  "about:blank",
  "https://trailing.slash/",
  "https://user:pass@site.example",
  "https://has.a.port:1234/and/path",
  "https://ümlauted.example",
  "https://has.a.fragment/#frag",
  "https://invalid.port:123456789",
  "blob:https://blob.example/guid-goes-here",
];

for (const invalid of invalidSerializations) {
  test(t => {
    assert_throws_js(TypeError, () => new Origin(invalid), "Constructor should throw TypeError for invalid origin.");
  }, `Origin constructor throws for '${invalid}'.`);

  test(t => {
    assert_equals(Origin.parse(invalid), null, "parse() should return null for invalid origin.");
  }, `Origin.parse returns null for '${invalid}'.`);
}

//
// Tuple origins:
//
const tupleSerializations = [
  "http://site.example",
  "https://site.example",
  "https://site.example:123",
  "http://sub.site.example",
  "https://sub.site.example",
  "https://sub.site.example:123",
  "https://xn--mlauted-m2a.example",
  "ftp://ftp.example",
  "ws://ws.example",
  "wss://wss.example",
];

for (const tuple of tupleSerializations) {
  test(t => {
    const origin = new Origin(tuple);
    assert_false(origin.opaque, "Origin should not be opaque.");
    assert_equals(origin.toJSON(), tuple, "toJSON() should return the serialized origin.");
  }, `Origin constructed from '${tuple}' is a tuple origin.`);

  test(t => {
    const origin = Origin.parse(tuple);
    assert_false(origin.opaque, "Origin should not be opaque.");
    assert_equals(origin.toJSON(), tuple, "toJSON() should return the serialized origin.");
  }, `Origin parsed from '${tuple}' is a tuple origin.`);

  test(t => {
    const a = new Origin(tuple);
    const b = Origin.parse(tuple);
    assert_true(a.isSameOrigin(b));
    assert_true(b.isSameOrigin(a));
  }, `Origins parsed and constructed from '${tuple}' are equivalent.`);
}
