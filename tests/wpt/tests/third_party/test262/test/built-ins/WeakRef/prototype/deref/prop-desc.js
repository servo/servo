// Copyright (C) 2019 Leo Balter. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-weak-ref.prototype.deref
description: >
  Property descriptor of WeakRef.prototype.deref
info: |
  17 ECMAScript Standard Built-in Objects:

  Every other data property described in clauses 18 through 26 and in Annex B.2
  has the attributes { [[Writable]]: true, [[Enumerable]]: false,
  [[Configurable]]: true } unless otherwise specified.
includes: [propertyHelper.js]
features: [WeakRef]
---*/

assert.sameValue(typeof WeakRef.prototype.deref, 'function');

verifyProperty(WeakRef.prototype, 'deref', {
  enumerable: false,
  writable: true,
  configurable: true
});
