// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
info: If x is not a string value, return x
esid: sec-performeval
es5id: 15.1.2.1_A1.1_T2
description: Checking all objects
---*/

var x = {};
assert.sameValue((0,eval)(x), x, 'ordinary object');

x = new Number(1);
assert.sameValue((0,eval)(x), x, 'Number object');

x = new Boolean(true);
assert.sameValue((0,eval)(x), x, 'Boolean object');

x = new String("1+1");
assert.sameValue((0,eval)(x), x, 'String object');
