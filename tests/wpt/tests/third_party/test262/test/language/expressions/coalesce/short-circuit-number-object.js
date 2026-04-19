// Copyright (C) 2019 Leo Balter. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: >
    Short circuit if the CoalesceExpressionHead is not undefined or null (object)
esid: sec-conditional-operator
info: |
    ConditionalExpression :
        ShortCircuitExpression
        ShortCircuitExpression ? AssignmentExpression : AssignmentExpression

    ShortCircuitExpression :
        LogicalORExpression
        CoalesceExpression

    CoalesceExpression :
        CoalesceExpressionHead ?? BitwiseORExpression

    CoalesceExpressionHead :
        CoalesceExpression
        BitwiseORExpression

    Runtime Semantics: Evaluation

    CoalesceExpression:CoalesceExpressionHead??BitwiseORExpression

    1. Let lref be the result of evaluating CoalesceExpressionHead.
    2. Let lval be ? GetValue(lref).
    3. If lval is undefined or null,
        a. Let rref be the result of evaluating BitwiseORExpression.
        b. Return ? GetValue(rref).
    4. Otherwise, return lval.
features: [coalesce-expression]
---*/

var x;
var obj = {
    toString() {
        return null;
    },
    valueOf() {
        return null;
    }
};

x = undefined;
x = obj ?? 1;
assert.sameValue(x, obj, 'obj ?? 1');

x = undefined;
x = obj ?? null;
assert.sameValue(x, obj, 'obj ?? null');

x = undefined;
x = obj ?? undefined;
assert.sameValue(x, obj, 'obj ?? undefined');

x = undefined;
x = obj ?? null ?? undefined;
assert.sameValue(x, obj, 'obj ?? null ?? undefined');

x = undefined;
x = obj ?? undefined ?? null;
assert.sameValue(x, obj, 'obj ?? undefined ?? null');

x = undefined;
x = obj ?? null ?? null;
assert.sameValue(x, obj, 'obj ?? null ?? null');

x = undefined;
x = obj ?? undefined ?? undefined;
assert.sameValue(x, obj, 'obj ?? null ?? null');

x = undefined;
x = null ?? obj ?? null;
assert.sameValue(x, obj, 'null ?? obj ?? null');

x = undefined;
x = null ?? obj ?? undefined;
assert.sameValue(x, obj, 'null ?? obj ?? undefined');

x = undefined;
x = undefined ?? obj ?? null;
assert.sameValue(x, obj, 'undefined ?? obj ?? null');

x = undefined;
x = undefined ?? obj ?? undefined;
assert.sameValue(x, obj, 'undefined ?? obj ?? undefined');
