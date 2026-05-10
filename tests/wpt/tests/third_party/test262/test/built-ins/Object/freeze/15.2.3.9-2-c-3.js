// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.9-2-c-3
description: >
    Object.freeze - The [[Configurable]] attribute of all own data
    property of 'O' is set to false while other attributes are
    unchanged
includes: [propertyHelper.js]
---*/

var obj = {};
var resultSetFun = false;

Object.defineProperty(obj, "foo1", {
  value: 10,
  writable: false,
  enumerable: true,
  configurable: true
});

function get_func() {
  return 10;
}

function set_func() {
  resultSetFun = true;
}

Object.defineProperty(obj, "foo2", {
  get: get_func,
  set: set_func,
  enumerable: true,
  configurable: true
});

Object.freeze(obj);

verifyProperty(obj, "foo2", {
  configurable: false,
});

verifyEqualTo(obj, "foo2", 10);

obj.foo2 = 12;
if (!resultSetFun) {
  throw new Test262Error('Expected obj["foo2"] set() to be called, but was not.');
}

verifyProperty(obj, "foo2", {
  enumerable: true,
  configurable: false,
});

var desc2 = Object.getOwnPropertyDescriptor(obj, "foo2");
if (desc2.writable) {
  throw new Test262Error('Expected obj["foo2"] to be non-writable, non-configurable; actually ' + JSON.stringify(desc2));
}

verifyProperty(obj, "foo1", {
  value: 10,
  writable: false,
  enumerable: true,
  configurable: false,
});
