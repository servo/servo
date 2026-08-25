// Copyright 2020 Jeff Walden, Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-isstructurallyvalidlanguagetag
description: >
  Verifies that just as duplicate variants in a tag ("en-emodeng-emodeng") make
  the tag structurally invalid, so too do duplicate variants in the tlang
  component of an otherwise structurally valid tag ("de-t-emodeng-emodeng"),
  make it structurally invalid.
info: |
  if a `transformed_extensions` component that contains a `tlang` component is
  present, then
    the `tlang` component contains no duplicate `unicode_variant_subtag`
    subtags.
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
// contain duplicate variants to be structurally valid.  This restriction also
// applies within the |tlang| component (indicating the source locale from which
// relevant content was transformed) of a broader language tag.

// Direct matches are rejected.
mustReject("de-t-en-emodeng-emodeng");
// Case-insensitive matches are also rejected.
mustReject("de-t-en-Emodeng-emodeng");
// ...and in either order.
mustReject("de-t-en-emodeng-Emodeng");

// Repeat the above tests with additional variants interspersed at each point
// for completeness.
mustReject("de-t-en-variant-emodeng-emodeng");
mustReject("de-t-en-variant-Emodeng-emodeng");
mustReject("de-t-en-variant-emodeng-Emodeng");
mustReject("de-t-en-emodeng-variant-emodeng");
mustReject("de-t-en-Emodeng-variant-emodeng");
mustReject("de-t-en-emodeng-variant-Emodeng");
mustReject("de-t-en-emodeng-emodeng-variant");
mustReject("de-t-en-Emodeng-emodeng-variant");
mustReject("de-t-en-emodeng-Emodeng-variant");
