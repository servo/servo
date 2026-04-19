// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.6-4-111
description: >
    Object.defineProperty  - 'name' and 'desc' are accessor
    properties, name.[[Set]] is present and desc.[[Set]] is undefined
    (8.12.9 step 12)
---*/

var obj = {};

function getFunc() {
  return 10;
}

function setFunc(value) {
  obj.setVerifyHelpProp = value;
}

Object.defineProperty(obj, "foo", {
  get: getFunc,
  set: setFunc,
  enumerable: true,
  configurable: true
});

Object.defineProperty(obj, "foo", {
  set: undefined,
  get: getFunc
});


var desc = Object.getOwnPropertyDescriptor(obj, "foo");

assert(obj.hasOwnProperty("foo"), 'obj.hasOwnProperty("foo") !== true');
assert.sameValue(typeof(desc.set), "undefined", 'typeof (desc.set)');
