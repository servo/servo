// Copyright (C) 2019 Leo Balter. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-properties-of-the-weak-ref-prototype-object
description: WeakRef.prototype.constructor
info: |
  WeakRef.prototype.constructor

  Normative Optional

  The initial value of WeakRef.prototype.constructor is the intrinsic object %WeakRef%.

  This property has the attributes { [[Writable]]: false, [[Enumerable]]: false, [[Configurable]]: true }.

  This section is to be treated identically to the "Annex B" of ECMA-262, but to be written in-line with the main specification.
includes: [propertyHelper.js]
features: [WeakRef]
---*/

var actual = WeakRef.prototype.hasOwnProperty('constructor');

// If implemented, it should conform to the spec text
if (actual) {
  verifyProperty(WeakRef.prototype, 'constructor', {
    value: WeakRef,
    writable: true,
    enumerable: false,
    configurable: true
  });
}
