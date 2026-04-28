// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-setintegritylevel
description: Object.seal - non-enumerable own property of 'O' is sealed
includes: [propertyHelper.js]
---*/

var obj = {};

Object.defineProperty(obj, "foo", {
  value: 10,
  enumerable: false,
  configurable: true
});

assert(Object.isExtensible(obj));
Object.seal(obj);

verifyProperty(obj, "foo", {
  configurable: false,
});
