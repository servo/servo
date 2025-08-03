// META: title=`Origin.fromURL()`

//
// URLs with opaque origins:
//
const opaqueURLs = [
  "about:blank",
  "data:text/plain,opaque",
  "weird-protocol:whatever",
  "weird-hierarchical-protocol://host/path?etc",
  "blob:weird-protocol:whatever",
  "blob:weird-hierarchical-protocol://host/path?etc",
];
for (const opaque of opaqueURLs) {
  test(t => {
    const origin = Origin.fromURL(opaque);
    assert_true(origin.opaque, "Origin should be opaque.");
    assert_equals(origin.toJSON(), "null", "toJSON() should return the serialized origin.");
  }, `Origin.fromURL for opaque URL as string '${opaque}'.`);

  test(t => {
    const origin = Origin.fromURL(new URL(opaque));
    assert_true(origin.opaque, "Origin should be opaque.");
    assert_equals(origin.toJSON(), "null", "toJSON() should return the serialized origin.");
  }, `Origin.fromURL for opaque URL as URL '${opaque}'.`);
}

//
// Invalid serializations:
//
const invalidSerializations = [
  "",
  "invalid",
];

for (const invalid of invalidSerializations) {
  test(t => {
    assert_equals(null, Origin.fromURL(invalid));
  }, `Origin.fromURL returns null for '${invalid}'.`);
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
  "https://trailing.slash/",
  "https://user:pass@site.example",
  "https://has.a.port:1234/and/path",
  "https://ümlauted.example",
  "file:///path/to/a/file.txt",
  "blob:https://example.com/some-guid",
  "ftp://example.com/",
  "https://example.com/path?query#fragment",
  "https://127.0.0.1/",
  "https://[::1]/",
];

for (const tuple of tupleSerializations) {
  test(t => {
    const origin = Origin.fromURL(tuple);
    assert_false(origin.opaque, "Origin should not be opaque.");
    assert_equals(origin.toJSON(), (new URL(tuple)).origin, "toJSON() should return the serialized origin.");
  }, `Origin constructed from '${tuple}' is a tuple origin.`);
}
