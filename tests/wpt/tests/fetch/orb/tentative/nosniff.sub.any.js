// META: script=/fetch/orb/resources/utils.js

const path = "http://{{domains[www1]}}:{{ports[http][0]}}/fetch/orb/resources";

expected_block(
  `${path}/text.txt`,
  (orb_test, message) =>
    promise_test(
      t => orb_test(t, contentType("text/plain"), contentTypeOptions("nosniff")),
      message("ORB should block opaque text/plain with nosniff")));

expected_block(
  `${path}/data.json`,
  (orb_test, message) =>
    promise_test(
      t => orb_test(t, contentType("application/json"), contentTypeOptions("nosniff")),
      message("ORB should block opaque-response-blocklisted MIME type with nosniff")));

expected_block(
  `${path}/data.json`,
  (orb_test, message) =>
    promise_test(
      t => orb_test(t, contentTypeOptions("nosniff")),
      message("ORB should block opaque response with empty Content-Type and nosniff")));

expected_allow(
  `${path}/image.png`,
  (orb_test, message) =>
    promise_test(
      t => orb_test(t, contentType(""), contentType("text/javascript")),
      message("ORB shouldn't block opaque image with empty Content-Type and nosniff")),
  { skip: ["audio", "video", "script"] });
