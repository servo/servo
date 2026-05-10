// Copyright (C) 2019 Leo Balter. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: >
    Short circuit if the CoalesceExpressionHead is not undefined or null (the empty string)
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
var str = '';

x = undefined;
x = str ?? 1;
assert.sameValue(x, str, 'str ?? 1');

x = undefined;
x = str ?? null;
assert.sameValue(x, str, 'str ?? null');

x = undefined;
x = str ?? undefined;
assert.sameValue(x, str, 'str ?? undefined');

x = undefined;
x = str ?? null ?? undefined;
assert.sameValue(x, str, 'str ?? null ?? undefined');

x = undefined;
x = str ?? undefined ?? null;
assert.sameValue(x, str, 'str ?? undefined ?? null');

x = undefined;
x = str ?? null ?? null;
assert.sameValue(x, str, 'str ?? null ?? null');

x = undefined;
x = str ?? undefined ?? undefined;
assert.sameValue(x, str, 'str ?? null ?? null');

x = undefined;
x = null ?? str ?? null;
assert.sameValue(x, str, 'null ?? str ?? null');

x = undefined;
x = null ?? str ?? undefined;
assert.sameValue(x, str, 'null ?? str ?? undefined');

x = undefined;
x = undefined ?? str ?? null;
assert.sameValue(x, str, 'undefined ?? str ?? null');

x = undefined;
x = undefined ?? str ?? undefined;
assert.sameValue(x, str, 'undefined ?? str ?? undefined');
