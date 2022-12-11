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
  "ORB should block opaque application/json"
);

promise_test(async () => {
  fetchORB(`${path}/image.png`, null, contentType("image/png"));
}, "ORB shouldn't block opaque image/png");

promise_test(async () => {
  await fetchORB(`${path}/script.js`, null, contentType("text/javascript"));
}, "ORB shouldn't block opaque text/javascript");
