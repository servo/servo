// META: script=/fetch/orb/resources/utils.js

const path = "http://{{domains[www1]}}:{{ports[http][0]}}/fetch/orb/resources";

expected_block(
  `${path}/font.ttf`,
  (orb_test, message) =>
    promise_test(
      t => orb_test(t, contentType("font/ttf")),
      message("ORB should block opaque font/ttf")));

expected_block(
  `${path}/text.txt`,
  (orb_test, message) =>
    promise_test(
      t => orb_test(t, contentType("text/plain")),
      message("ORB should block opaque text/plain")));

expected_block(
  `${path}/data.json`,
  (orb_test, message) =>
    promise_test(
      t => orb_test(t, contentType("application/json")),
      message("ORB should block opaque application/json (non-empty)")));

expected_block(
  `${path}/empty.json`,
  (orb_test, message) =>
    promise_test(
      t => orb_test(t, contentType("application/json")),
      message("ORB should block opaque application/json (empty)")));

expected_block(
  `${path}/data_non_ascii.json`,
  (orb_test, message) =>
    promise_test(
      t => orb_test(t, contentType("application/json")),
      message("ORB should block opaque application/json which contains non ascii characters")));

expected_allow(
  `${path}/image.png`,
  (orb_test, message) =>
    promise_test(
      t => orb_test(t, contentType("image/png")),
      message("ORB shouldn't block opaque image/png")),
  { skip: ["audio", "video", "script"] });

expected_allow(
  `${path}/script.js`,
  (orb_test, message) =>
    promise_test(
      t => orb_test(t, contentType("text/javascript")),
      message("ORB shouldn't block opaque text/javascript")),
  { skip: ["image", "audio", "video"] });

// Test javascript validation can correctly decode the content with BOM.
expected_allow(
  `${path}/script-utf16-bom.js`,
  (orb_test, message) =>
    promise_test(
      t => orb_test(t, contentType("application/json")),
      message("ORB shouldn't block opaque text/javascript (utf16 encoded with BOM)")),
  { skip: ["image", "audio", "video"] });

// Test javascript validation can correctly decode the content with the http charset hint.
expected_allow(
  `${path}/script-utf16-without-bom.js`,
  (orb_test, message) =>
    promise_test(
      t => orb_test(t, contentType("application/json; charset=utf-16")),
      message("ORB shouldn't block opaque text/javascript (utf16 encoded without BOM but charset is provided in content-type)")),
  { skip: ["image", "audio", "video"] });

// Test javascript validation can correctly decode the content for iso-8559-1 (fallback decoder in Firefox).
expected_allow(
  `${path}/script-iso-8559-1.js`,
  (orb_test, message) =>
    promise_test(
      t => orb_test(t, contentType("application/json")),
      message("ORB shouldn't block opaque text/javascript (iso-8559-1 encoded)")),
  { skip: ["image", "audio", "video"] });

// Test javascript validation can correctly parse asm.js.
expected_allow(
  `${path}/script-asm-js-valid.js`,
  (orb_test, message) =>
    promise_test(
      t => orb_test(t, contentType("application/json")),
      message("ORB shouldn't block text/javascript with valid asm.js")),
  { skip: ["image", "audio", "video"] });

// Test javascript validation can correctly parse invalid asm.js with valid JS syntax.
expected_allow(
  `${path}/script-asm-js-invalid.js`,
  (orb_test, message) =>
    promise_test(
      t => orb_test(t, contentType("application/json")),
      message("ORB shouldn't block text/javascript with invalid asm.js")),
  { skip: ["image", "audio", "video"] });
