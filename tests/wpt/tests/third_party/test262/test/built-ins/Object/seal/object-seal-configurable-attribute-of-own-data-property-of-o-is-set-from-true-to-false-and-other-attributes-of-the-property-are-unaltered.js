// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-setintegritylevel
description: >
    Object.seal - the [[Configurable]] attribute of own data property
    of 'O' is set from true to false and other attributes of the
    property are unaltered
includes: [propertyHelper.js]
---*/

var obj = {};

Object.defineProperty(obj, "foo", {
  value: 10,
  writable: true,
  enumerable: true,
  configurable: true
});
var preCheck = Object.isExtensible(obj);
Object.seal(obj);

if (!preCheck) {
  throw new Test262Error('Expected preCheck to be true, actually ' + preCheck);
}

verifyProperty(obj, "foo", {
  value: 10,
  writable: true,
  enumerable: true,
  configurable: false,
});
