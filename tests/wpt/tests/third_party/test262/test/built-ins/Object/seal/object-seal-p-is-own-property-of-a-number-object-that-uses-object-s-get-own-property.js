// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-setintegritylevel
description: >
    Object.seal - 'P' is own property of a Number object that uses
    Object's [[GetOwnProperty]]
includes: [propertyHelper.js]
---*/

var obj = new Number(-1);

obj.foo = 10;

assert(Object.isExtensible(obj));
Object.seal(obj);

verifyProperty(obj, "foo", {
  value: 10,
  configurable: false,
});
