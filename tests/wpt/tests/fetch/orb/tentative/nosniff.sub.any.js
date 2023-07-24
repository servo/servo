// META: script=/fetch/orb/resources/utils.js

const path = "http://{{domains[www1]}}:{{ports[http][0]}}/fetch/orb/resources";

promise_test(
  t =>
    promise_rejects_js(
      t,
      TypeError,
      fetchORB(
        `${path}/text.txt`,
        null,
        contentType("text/plain"),
        contentTypeOptions("nosniff")
      )
    ),
  "ORB should block opaque text/plain with nosniff"
);

promise_test(
  t =>
    promise_rejects_js(
      t,
      TypeError,
      fetchORB(
        `${path}/data.json`,
        null,
        contentType("application/json"),
        contentTypeOptions("nosniff")
      )
    ),
  "ORB should block opaque-response-blocklisted MIME type with nosniff"
);

promise_test(
  t =>
    promise_rejects_js(
      t,
      TypeError,
      fetchORB(
        `${path}/data.json`,
        null,
        contentType(""),
        contentTypeOptions("nosniff")
      )
    ),
  "ORB should block opaque response with empty Content-Type and nosniff"
);

promise_test(
  () =>
    fetchORB(
      `${path}/image.png`,
      null,
      contentType(""),
      contentTypeOptions("nosniff")
    ),
  "ORB shouldn't block opaque image with empty Content-Type and nosniff"
);
