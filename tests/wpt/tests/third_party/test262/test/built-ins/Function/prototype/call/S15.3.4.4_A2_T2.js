// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: The length property of the call method is 1
es5id: 15.3.4.4_A2_T2
description: Checking f.call.length, where f is new Function
---*/

var f = new Function;

assert.sameValue(typeof f.call, "function", 'The value of `typeof f.call` is expected to be "function"');
assert.notSameValue(typeof f.call.length, "undefined", 'The value of typeof f.call.length is not "undefined"');
assert.sameValue(f.call.length, 1, 'The value of f.call.length is expected to be 1');
