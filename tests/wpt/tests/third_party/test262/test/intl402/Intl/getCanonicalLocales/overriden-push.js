// Copyright 2016 Mozilla Corporation. All rights reserved.
// This code is governed by the license found in the LICENSE file.

/*---
esid: sec-intl.getcanonicallocales
description: Tests the getCanonicalLocales function for overridden Array.push().
info: |
  8.2.1 Intl.getCanonicalLocales (locales)
  1. Let ll be ? CanonicalizeLocaleList(locales).
  2. Return CreateArrayFromList(ll).
includes: [compareArray.js]
---*/

Array.prototype.push = function() { throw 42; };

// must not throw 42, might if push is used
var arr = Intl.getCanonicalLocales(["en-US"]);

assert.compareArray(arr, ["en-US"]);
