// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.5-4-315
description: >
    Object.create - all properties in 'Properties' are enumerable
    (data property and accessor property) (15.2.3.7 step 7)
includes: [propertyHelper.js]
---*/

var newObj = {};

function getFunc() {
  return 10;
}

function setFunc(value) {
  newObj.setVerifyHelpProp = value;
}

newObj = Object.create({}, {
  foo1: {
    value: 200,
    enumerable: true,
    writable: true,
    configurable: true
  },
  foo2: {
    get: getFunc,
    set: setFunc,
    enumerable: true,
    configurable: true
  }
});

verifyProperty(newObj, "foo1", {
  value: 200,
  writable: true,
  enumerable: true,
  configurable: true,
});

verifyEqualTo(newObj, "foo2", getFunc());

verifyWritable(newObj, "foo2", "setVerifyHelpProp");

verifyProperty(newObj, "foo2", {
  enumerable: true,
  configurable: true,
});
