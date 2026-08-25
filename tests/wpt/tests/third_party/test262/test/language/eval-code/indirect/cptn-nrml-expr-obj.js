// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
info: |
    If Result(3).type is normal and its completion value is a value V,
    then return the value V
esid: sec-performeval
es5id: 15.1.2.1_A3.1_T2
description: Expression statement. Eval return object value
---*/

var x = {};
var y;

assert.sameValue((0,eval)("y = x"),  x, 'AssignmentExpression');

assert.sameValue((0,eval)("x"), x, 'IdentifierReference');
