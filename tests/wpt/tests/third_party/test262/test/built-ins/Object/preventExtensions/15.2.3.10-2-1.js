// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.10-2-1
description: >
    Object.preventExtensions - repeated calls to preventExtensions
    have no side effects
---*/

var obj = {};
var testResult1 = true;
var testResult2 = true;

var preCheck = Object.isExtensible(obj);

Object.preventExtensions(obj);
testResult1 = Object.isExtensible(obj);
Object.preventExtensions(obj);
testResult2 = Object.isExtensible(obj);

assert(preCheck, 'preCheck !== true');
assert.sameValue(testResult1, false, 'testResult1');
assert.sameValue(testResult2, false, 'testResult2');
