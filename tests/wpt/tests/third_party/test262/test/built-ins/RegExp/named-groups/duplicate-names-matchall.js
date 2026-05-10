// Copyright 2022 Igalia S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: String.prototype.search behavior with duplicate named capture groups
esid: prod-GroupSpecifier
includes: [compareArray.js,compareIterator.js]
features: [regexp-duplicate-named-groups]
---*/

function matchesIterator(iterator, expected) {
  assert.compareIterator(iterator, expected.map(e => {
    return v => assert.compareArray(v, e);
  }));
}

matchesIterator("bab".matchAll(/(?<x>a)|(?<x>b)/g),
  [
    ["b", undefined, "b"],
    ["a", "a", undefined],
    ["b", undefined, "b"],
  ]);
matchesIterator("bab".matchAll(/(?<x>b)|(?<x>a)/g),
  [
    ["b", "b", undefined],
    ["a", undefined, "a"],
    ["b", "b", undefined],
  ]);
