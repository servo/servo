// Copyright (C) 2019 Leo Balter. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: >
    Short circuit if the CoalesceExpressionHead is not undefined or null (Symbol)
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
var s = Symbol();

x = undefined;
x = s ?? 1;
assert.sameValue(x, s, 's ?? null');

x = undefined;
x = s ?? null;
assert.sameValue(x, s, 's ?? null');

x = undefined;
x = s ?? undefined;
assert.sameValue(x, s, 's ?? undefined');

x = undefined;
x = s ?? null ?? undefined;
assert.sameValue(x, s, 's ?? null ?? undefined');

x = undefined;
x = s ?? undefined ?? null;
assert.sameValue(x, s, 's ?? undefined ?? null');

x = undefined;
x = s ?? null ?? null;
assert.sameValue(x, s, 's ?? null ?? null');

x = undefined;
x = s ?? undefined ?? undefined;
assert.sameValue(x, s, 's ?? null ?? null');

x = undefined;
x = null ?? s ?? null;
assert.sameValue(x, s, 'null ?? s ?? null');

x = undefined;
x = null ?? s ?? undefined;
assert.sameValue(x, s, 'null ?? s ?? undefined');

x = undefined;
x = undefined ?? s ?? null;
assert.sameValue(x, s, 'undefined ?? s ?? null');

x = undefined;
x = undefined ?? s ?? undefined;
assert.sameValue(x, s, 'undefined ?? s ?? undefined');
