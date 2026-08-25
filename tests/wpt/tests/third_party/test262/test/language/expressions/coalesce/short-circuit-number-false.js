// Copyright (C) 2019 Leo Balter. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: >
    Short circuit if the CoalesceExpressionHead is not undefined or null (false)
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

x = undefined;
x = false ?? 1;
assert.sameValue(x, false, 'false ?? 1');

x = undefined;
x = false ?? null;
assert.sameValue(x, false, 'false ?? null');

x = undefined;
x = false ?? undefined;
assert.sameValue(x, false, 'false ?? undefined');

x = undefined;
x = false ?? null ?? undefined;
assert.sameValue(x, false, 'false ?? null ?? undefined');

x = undefined;
x = false ?? undefined ?? null;
assert.sameValue(x, false, 'false ?? undefined ?? null');

x = undefined;
x = false ?? null ?? null;
assert.sameValue(x, false, 'false ?? null ?? null');

x = undefined;
x = false ?? undefined ?? undefined;
assert.sameValue(x, false, 'false ?? null ?? null');

x = undefined;
x = null ?? false ?? null;
assert.sameValue(x, false, 'null ?? false ?? null');

x = undefined;
x = null ?? false ?? undefined;
assert.sameValue(x, false, 'null ?? false ?? undefined');

x = undefined;
x = undefined ?? false ?? null;
assert.sameValue(x, false, 'undefined ?? false ?? null');

x = undefined;
x = undefined ?? false ?? undefined;
assert.sameValue(x, false, 'undefined ?? false ?? undefined');
