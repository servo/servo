// Copyright (C) 2019 Leo Balter. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: >
    Short circuit if the CoalesceExpressionHead is not undefined or null (0)
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
x = 0 ?? 1;
assert.sameValue(x, 0, '0 ?? 1');

x = undefined;
x = 0 ?? null;
assert.sameValue(x, 0, '0 ?? null');

x = undefined;
x = 0 ?? undefined;
assert.sameValue(x, 0, '0 ?? undefined');

x = undefined;
x = 0 ?? null ?? undefined;
assert.sameValue(x, 0, '0 ?? null ?? undefined');

x = undefined;
x = 0 ?? undefined ?? null;
assert.sameValue(x, 0, '0 ?? undefined ?? null');

x = undefined;
x = 0 ?? null ?? null;
assert.sameValue(x, 0, '0 ?? null ?? null');

x = undefined;
x = 0 ?? undefined ?? undefined;
assert.sameValue(x, 0, '0 ?? null ?? null');

x = undefined;
x = null ?? 0 ?? null;
assert.sameValue(x, 0, 'null ?? 0 ?? null');

x = undefined;
x = null ?? 0 ?? undefined;
assert.sameValue(x, 0, 'null ?? 0 ?? undefined');

x = undefined;
x = undefined ?? 0 ?? null;
assert.sameValue(x, 0, 'undefined ?? 0 ?? null');

x = undefined;
x = undefined ?? 0 ?? undefined;
assert.sameValue(x, 0, 'undefined ?? 0 ?? undefined');
