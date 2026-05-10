// Copyright (C) 2019 Leo Balter. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: >
    Abrupt completions are also a Short circuit and prevent evaluation of the right-side expressions
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
function poison() {
    throw new Test262Error('poison handled');
}

function morePoison() {
    throw 'poison!!!!';
}

x = undefined;
assert.throws(Test262Error, function() {
    undefined ?? poison() ?? morePoison();
}, 'undefined ?? poison() ?? morePoison();');

x = undefined;
assert.throws(Test262Error, function() {
    null ?? poison() ?? morePoison();
}, 'null ?? poison() ?? morePoison();');

assert.throws(Test262Error, function() {
    poison() ?? morePoison();
}, 'poison() ?? morePoison();');
