// Copyright (C) 2019 Leo Balter. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: >
    ShortCircuitExpression in the Conditional Expression (? :)
esid: sec-conditional-operator
info: |
    ShortCircuitExpression :
        LogicalORExpression
        CoalesceExpression

    CoalesceExpression :
        CoalesceExpressionHead ?? BitwiseORExpression

    CoalesceExpressionHead :
        CoalesceExpression
        BitwiseORExpression

    ConditionalExpression :
        ShortCircuitExpression
        ShortCircuitExpression ? AssignmentExpression : AssignmentExpression
features: [coalesce-expression]
---*/

var x;

x = undefined ?? true ? 0 : 42;
assert.sameValue(x, 0, 'undefined ?? true ? 0 : 42');

x = undefined;
x = null ?? true ? 0 : 42;
assert.sameValue(x, 0, 'null ?? true ? 0 : 42');

x = undefined;
x = undefined ?? false ? 0 : 42;
assert.sameValue(x, 42, 'undefined ?? false ? 0 : 42');

x = undefined;
x = null ?? false ? 0 : 42;
assert.sameValue(x, 42, 'null ?? false ? 0 : 42');

x = undefined;
x = false ?? true ? 0 : 42;
assert.sameValue(x, 42, 'false ?? true ? 0 : 42');

x = undefined;
x = 0 ?? true ? 0 : 42;
assert.sameValue(x, 42, '0 ?? true ? 0 : 42');

x = undefined;
x = 1 ?? false ? 0 : 42;
assert.sameValue(x, 0, '1 ?? false ? 0 : 42');

x = undefined;
x = true ?? false ? 0 : 42;
assert.sameValue(x, 0, 'true ?? false ? 0 : 42');

x = undefined;
x = true ?? true ? 0 : 42;
assert.sameValue(x, 0, 'true ?? true ? 0 : 42');

x = undefined;
x = '' ?? true ? 0 : 42;
assert.sameValue(x, 42, '"" ?? true ? 0 : 42');

x = undefined;
x = Symbol() ?? false ? 0 : 42;
assert.sameValue(x, 0, 'Symbol() ?? false ? 0 : 42');

x = undefined;
x = {} ?? false ? 0 : 42;
assert.sameValue(x, 0, 'object ?? false ? 0 : 42');
