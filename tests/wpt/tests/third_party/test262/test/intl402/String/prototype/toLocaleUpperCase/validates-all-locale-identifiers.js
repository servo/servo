// Copyright (C) 2025 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sup-string.prototype.tolocaleuppercase
description: >
  All locale identifiers are validated, not just the first one.
info: |
  String.prototype.toLocaleUpperCase ( [ locales ] )
    ...
    3. Return ? TransformCase(S, locales, upper).

  TransformCase ( S, locales, targetCase )
    1. Let requestedLocales be ? CanonicalizeLocaleList(locales).
    ...
---*/

var locales = [
  "en-US",
  "this is not a valid locale",
];

assert.throws(RangeError, function() {
  "".toLocaleUpperCase(locales);
});
