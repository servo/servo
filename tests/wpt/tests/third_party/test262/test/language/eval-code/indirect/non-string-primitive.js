// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
info: If x is not a string value, return x
esid: sec-performeval
es5id: 15.1.2.1_A1.1_T1
description: Checking all primitives
---*/

var x = 1;
assert.sameValue((0,eval)(x), x, 'Reference');

assert.sameValue((0,eval)(1), 1, 'number');

assert.sameValue((0,eval)(true), true, 'boolean');

assert.sameValue((0,eval)(null), null, 'null');

assert.sameValue((0,eval)(undefined), undefined, 'undefined');
