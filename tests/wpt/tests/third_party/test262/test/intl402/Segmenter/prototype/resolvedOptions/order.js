// Copyright 2018 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-Intl.Segmenter.prototype.resolvedOptions
description: Verifies the property order for the object returned by resolvedOptions().
features: [Intl.Segmenter]
---*/

const options = new Intl.Segmenter([], {
  "granularity": "word",
}).resolvedOptions();

const expected = [
  "locale",
  "granularity",
];

const actual = Object.getOwnPropertyNames(options);

// Ensure all expected items are in actual and also allow other properties
// implemented in new proposals.
assert(actual.indexOf("locale") > -1, "\"locale\" is present");
for (var i = 1; i < expected.length; i++) {
  // Ensure the order as expected but allow additional new property in between
  assert(actual.indexOf(expected[i-1]) < actual.indexOf(expected[i]), `"${expected[i-1]}" precedes "${expected[i]}"`);
}
