// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.4-4-50
description: >
    Object.getOwnPropertyNames - non-enumerable own property of 'O' is
    pushed into the returned Array
---*/

var obj = {};

Object.defineProperty(obj, "nonEnumerableProp", {
  value: 10,
  enumerable: false,
  configurable: true
});

var result = Object.getOwnPropertyNames(obj);

assert.sameValue(result[0], "nonEnumerableProp", 'result[0]');
