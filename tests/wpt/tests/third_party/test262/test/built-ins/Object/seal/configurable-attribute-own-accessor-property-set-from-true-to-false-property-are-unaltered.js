// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-setintegritylevel
description: >
    Object.seal - the [[Configurable]] attribute of own accessor
    property of 'O' is set from true to false and other attributes of
    the property are unaltered
includes: [propertyHelper.js]
---*/

var obj = {};
obj.variableForHelpVerify = "data";

function setFunc(value) {
  obj.variableForHelpVerify = value;
}

function getFunc() {
  return 10;
}
Object.defineProperty(obj, "foo", {
  get: getFunc,
  set: setFunc,
  enumerable: true,
  configurable: true
});
var preCheck = Object.isExtensible(obj);
Object.seal(obj);

if (!preCheck) {
  throw new Test262Error('Expected preCheck to be true, actually ' + preCheck);
}

verifyEqualTo(obj, "foo", getFunc());

verifyWritable(obj, "foo", "variableForHelpVerify");

verifyProperty(obj, "foo", {
  enumerable: true,
  configurable: false,
});
