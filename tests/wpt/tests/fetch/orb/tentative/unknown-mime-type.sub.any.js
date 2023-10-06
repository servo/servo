// META: script=/fetch/orb/resources/utils.js

const path = "http://{{domains[www1]}}:{{ports[http][0]}}/fetch/orb/resources";

expected_allow_fetch(
  `${path}/font.ttf`,
  (promise, message) =>
    promise_test(
      t => promise(t, contentType("")),
      message("ORB shouldn't block opaque failed missing MIME type (font/ttf)")));

expected_allow_fetch(
  `${path}/text.ttf`,
  (promise, message) =>
    promise_test(
      t => promise(t, contentType("")),
      message("ORB shouldn't block opaque failed missing MIME type (text/plain)")));

expected_allow_fetch(
  `${path}/data.json`,
  (promise, message) =>
    promise_test(
      t => promise(t, contentType("")),
      message("ORB shouldn't block opaque failed missing MIME type (application/json)")));

expected_allow(
  `${path}/image.png`,
  (promise, message) =>
    promise_test(
      t => promise(t, contentType("")),
      message("ORB shouldn't block opaque failed missing MIME type (image/png)")),
  { skip: ["audio", "video", "script"] });

expected_allow(
  `${path}/script.js`,
  (promise, message) =>
    promise_test(
      t => promise(t, contentType("")),
      message("ORB shouldn't block opaque failed missing MIME type (text/javascript)")),
      { skip: ["image", "audio", "video"] });
