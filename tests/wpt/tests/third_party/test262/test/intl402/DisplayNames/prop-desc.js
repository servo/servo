// Copyright (C) 2019 Leo Balter. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-Intl.DisplayNames
description: >
  Property descriptor of Intl.DisplayNames
info: |
  17 ECMAScript Standard Built-in Objects:

  Every other data property described in clauses 18 through 26 and in Annex B.2
  has the attributes { [[Writable]]: true, [[Enumerable]]: false,
  [[Configurable]]: true } unless otherwise specified.
includes: [propertyHelper.js]
features: [Intl.DisplayNames]
---*/

assert.sameValue(typeof Intl.DisplayNames, "function", "`typeof Intl.DisplayNames` is `'function'`");

verifyProperty(Intl, "DisplayNames", {
  writable: true,
  enumerable: false,
  configurable: true,
});
