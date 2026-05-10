// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-setintegritylevel
description: >
    Object.seal - all own properties of 'O' are already
    non-configurable
includes: [propertyHelper.js]
---*/

var obj = {};
obj.variableForHelpVerify = "data";

Object.defineProperty(obj, "foo1", {
  value: 10,
  writable: true,
  enumerable: true,
  configurable: false
});

function set_func(value) {
  obj.variableForHelpVerify = value;
}

function get_func() {
  return 10;
}
Object.defineProperty(obj, "foo2", {
  get: get_func,
  set: set_func,
  enumerable: true,
  configurable: false
});

if (!Object.isExtensible(obj)) {
  throw new Test262Error('Expected obj to be extensible, actually ' + Object.isExtensible(obj));
}

Object.seal(obj);

if (Object.isExtensible(obj)) {
  throw new Test262Error('Expected obj NOT to be extensible, actually ' + Object.isExtensible(obj));
}

verifyProperty(obj, "foo1", {
  value: 10,
  writable: true,
  enumerable: true,
  configurable: false,
});

verifyEqualTo(obj, "foo2", get_func());

verifyWritable(obj, "foo2", "variableForHelpVerify");

verifyProperty(obj, "foo2", {
  enumerable: true,
  configurable: false,
});
