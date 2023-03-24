// META: script=/fetch/orb/resources/utils.js

const path = "http://{{domains[www1]}}:{{ports[http][0]}}/fetch/orb/resources";

promise_test(
  t =>
    promise_rejects_js(
      t,
      TypeError,
      fetchORB(`${path}/font.ttf`, null, contentType("font/ttf"))
    ),
  "ORB should block opaque font/ttf"
);

promise_test(
  t =>
    promise_rejects_js(
      t,
      TypeError,
      fetchORB(`${path}/text.txt`, null, contentType("text/plain"))
    ),
  "ORB should block opaque text/plain"
);

promise_test(
  t =>
    promise_rejects_js(
      t,
      TypeError,
      fetchORB(`${path}/data.json`, null, contentType("application/json"))
    ),
  "ORB should block opaque application/json (non-empty)"
);

promise_test(
  t =>
    promise_rejects_js(
      t,
      TypeError,
      fetchORB(`${path}/empty.json`, null, contentType("application/json"))
    ),
  "ORB should block opaque application/json (empty)"
);

promise_test(
  t =>
    promise_rejects_js(
      t,
      TypeError,
      fetchORB(`${path}/data_non_ascii.json`, null, contentType("application/json"))
    ),
  "ORB should block opaque application/json which contains non ascii characters"
);

promise_test(async () => {
  fetchORB(`${path}/image.png`, null, contentType("image/png"));
}, "ORB shouldn't block opaque image/png");

promise_test(async () => {
  await fetchORB(`${path}/script.js`, null, contentType("text/javascript"));
}, "ORB shouldn't block opaque text/javascript");

// Test javascript validation can correctly decode the content with BOM.
promise_test(async () => {
  await fetchORB(`${path}/script-utf16-bom.js`, null, contentType("application/json"));
}, "ORB shouldn't block opaque text/javascript (utf16 encoded with BOM)");

// Test javascript validation can correctly decode the content with the http charset hint.
promise_test(async () => {
  await fetchORB(`${path}/script-utf16-without-bom.js`, null, contentType("application/json; charset=utf-16"));
}, "ORB shouldn't block opaque text/javascript (utf16 encoded without BOM but charset is provided in content-type)");

// Test javascript validation can correctly decode the content for iso-8559-1 (fallback decoder in Firefox).
promise_test(async () => {
  await fetchORB(`${path}/script-iso-8559-1.js`, null, contentType("application/json"));
}, "ORB shouldn't block opaque text/javascript (iso-8559-1 encoded)");
