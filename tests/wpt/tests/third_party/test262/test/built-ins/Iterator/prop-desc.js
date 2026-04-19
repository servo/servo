// Copyright (C) 2020 Rick Waldron. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-iterator-constructor
description: >
  Property descriptor of Iterator
info: |
  The Iterator Constructor

  * is the initial value of the Iterator property of the global object.

  17 ECMAScript Standard Built-in Objects

  Every other data property described in clauses 18 through 26 and in Annex B.2
  has the attributes { [[Writable]]: true, [[Enumerable]]: false,
  [[Configurable]]: true } unless otherwise specified.
features: [globalThis, iterator-helpers]
includes: [propertyHelper.js]
---*/

verifyProperty(globalThis, 'Iterator', {
  value: Iterator,
  writable: true,
  enumerable: false,
  configurable: true,
});
