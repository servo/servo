// META: script=/fetch/orb/resources/utils.js

const path = "http://{{domains[www1]}}:{{ports[http][0]}}/fetch/orb/resources";

promise_test(
  t =>
    promise_rejects_js(
      t,
      TypeError,
      fetchORB(
        `${path}/data.json`,
        null,
        contentType("application/json"),
        "status(206)"
      )
    ),
  "ORB should block opaque-response-blocklisted MIME type with status 206"
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
        "status(302)"
      )
    ),
  "ORB should block opaque response with non-ok status"
);
