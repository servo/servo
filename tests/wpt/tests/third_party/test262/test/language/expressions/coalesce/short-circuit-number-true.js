// Copyright (C) 2019 Leo Balter. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: >
    Short circuit if the CoalesceExpressionHead is not undefined or null (true)
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
x = true ?? 1;
assert.sameValue(x, true, 'true ?? null');

x = undefined;
x = true ?? null;
assert.sameValue(x, true, 'true ?? null');

x = undefined;
x = true ?? undefined;
assert.sameValue(x, true, 'true ?? undefined');

x = undefined;
x = true ?? null ?? undefined;
assert.sameValue(x, true, 'true ?? null ?? undefined');

x = undefined;
x = true ?? undefined ?? null;
assert.sameValue(x, true, 'true ?? undefined ?? null');

x = undefined;
x = true ?? null ?? null;
assert.sameValue(x, true, 'true ?? null ?? null');

x = undefined;
x = true ?? undefined ?? undefined;
assert.sameValue(x, true, 'true ?? null ?? null');

x = undefined;
x = null ?? true ?? null;
assert.sameValue(x, true, 'null ?? true ?? null');

x = undefined;
x = null ?? true ?? undefined;
assert.sameValue(x, true, 'null ?? true ?? undefined');

x = undefined;
x = undefined ?? true ?? null;
assert.sameValue(x, true, 'undefined ?? true ?? null');

x = undefined;
x = undefined ?? true ?? undefined;
assert.sameValue(x, true, 'undefined ?? true ?? undefined');
