// META: script=/fetch/orb/resources/utils.js

const url =
  "http://{{domains[www1]}}:{{ports[http][0]}}/fetch/orb/resources/image.png";

expected_allow_fetch(
  url,
  (orb_test, message) =>
    promise_test(
      t => orb_test(t, header("Content-Range", "bytes 0-99/1010"), "slice(null,100)", "status(206)"),
      message("ORB shouldn't block opaque range of image/png starting at zero")),
  { headers: new Headers([["Range", "bytes=0-99"]]) });

expected_block_fetch(
  url,
  (orb_test, message) =>
    promise_test(
      t => orb_test(t, header("Content-Range", "bytes 10-99/1010"), "slice(10,100)", "status(206)"),
      message("ORB should block opaque range of image/png not starting at zero, that isn't subsequent")),
  { headers: new Headers([["Range", "bytes 10-99"]]) });
