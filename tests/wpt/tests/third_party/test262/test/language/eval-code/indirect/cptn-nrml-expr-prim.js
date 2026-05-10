// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
info: |
    If Result(3).type is normal and its completion value is a value V,
    then return the value V
esid: sec-performeval
es5id: 15.1.2.1_A3.1_T1
description: Expression statement. Eval return primitive value
---*/

var x;
assert.sameValue((0,eval)("x = 1"), 1, 'AssignmentExpression');

assert.sameValue((0,eval)("1"), 1, 'NumericLiteral');

assert.sameValue((0,eval)("'1'"), '1', 'StringLiteral');

x = 1;
assert.sameValue((0,eval)("++x"), 2, 'UpdateExpression');
