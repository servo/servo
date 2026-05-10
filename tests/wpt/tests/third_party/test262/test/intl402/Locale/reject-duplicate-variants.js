// Copyright 2020 Jeff Walden, Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-isstructurallyvalidlanguagetag
description: >
  Verifies that duplicate variants in a tag ("en-emodeng-emodeng") make the tag
  structurally invalid.
info: |
  the `unicode_language_id` within _locale_ contains no duplicate
  `unicode_variant_subtag` subtags
features: [Intl.Locale]
---*/

assert.sameValue(typeof Intl.Locale, "function");

function mustReject(tag) {
  assert.throws(RangeError, () => {
    // Direct matches are rejected.
    new Intl.Locale(tag);
  }, `tag "${tag}" must be considered structurally invalid`);
}

// BCP47 since forever, and ECMA-402 as consequence, do not consider tags that
// contain duplicate variants to be structurally valid.

// Direct matches are rejected.
mustReject("en-emodeng-emodeng");
// Case-insensitive matches are also rejected.
mustReject("en-Emodeng-emodeng");
// ...and in either order.
mustReject("en-emodeng-Emodeng");

// Repeat the above tests with additional variants interspersed at each point
// for completeness.
mustReject("en-variant-emodeng-emodeng");
mustReject("en-variant-Emodeng-emodeng");
mustReject("en-variant-emodeng-Emodeng");
mustReject("en-emodeng-variant-emodeng");
mustReject("en-Emodeng-variant-emodeng");
mustReject("en-emodeng-variant-Emodeng");
mustReject("en-emodeng-emodeng-variant");
mustReject("en-Emodeng-emodeng-variant");
mustReject("en-emodeng-Emodeng-variant");
