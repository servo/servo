// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-setintegritylevel
description: >
    Object.seal - the [[Configurable]] attribute of all own properties
    of 'O' are set from true to false and other attributes of the
    property are unaltered
includes: [propertyHelper.js]
---*/

var obj = {};
obj.variableForHelpVerify = "data";

Object.defineProperty(obj, "foo1", {
  value: 10,
  writable: true,
  enumerable: true,
  configurable: true
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
  configurable: true
});
var preCheck = Object.isExtensible(obj);
Object.seal(obj);

if (!preCheck) {
  throw new Test262Error('Expected preCheck to be true, actually ' + preCheck);
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
