// Copyright (C) 2019 Leo Balter. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: >
    If the CoalesceExpressionHead is undefined or null, follow return the right-side value.
    Otherwise, return the left-side value.
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

x = null ?? undefined ?? 42;
assert.sameValue(x, 42, 'null ?? undefined ?? 42');

x = undefined ?? null ?? 42;
assert.sameValue(x, 42, 'undefined ?? null ?? 42');

x = null ?? null ?? 42;
assert.sameValue(x, 42, 'null ?? null ?? 42');

x = undefined ?? undefined ?? 42;
assert.sameValue(x, 42, 'null ?? null ?? 42');
