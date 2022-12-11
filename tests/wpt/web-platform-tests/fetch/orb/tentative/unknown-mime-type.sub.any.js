// META: script=/fetch/orb/resources/utils.js

const path = "http://{{domains[www1]}}:{{ports[http][0]}}/fetch/orb/resources";

promise_test(
  () => fetchORB(`${path}/font.ttf`, null, contentType("")),
  "ORB shouldn't block opaque failed missing MIME type (font/ttf)"
);

promise_test(
  () => fetchORB(`${path}/text.txt`, null, contentType("")),
  "ORB shouldn't block opaque failed missing MIME type (text/plain)"
);

promise_test(
  t => fetchORB(`${path}/data.json`, null, contentType("")),
  "ORB shouldn't block opaque failed missing MIME type (application/json)"
);

promise_test(
  () => fetchORB(`${path}/image.png`, null, contentType("")),
  "ORB shouldn't block opaque failed missing MIME type (image/png)"
);

promise_test(
  () => fetchORB(`${path}/script.js`, null, contentType("")),
  "ORB shouldn't block opaque failed missing MIME type (text/javascript)"
);
