// Copyright (C) 2018 Leo Balter. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
description: >
    Dynamic Import receives an AssignmentExpression (MemberExpression TemplateLiteral)
esid: prod-ImportCall
info: |
    ImportCall [Yield]:
        import ( AssignmentExpression[+In, ?Yield] )

    AssignmentExpression[In, Yield, Await]:
        ConditionalExpression[?In, ?Yield, ?Await]
        [+Yield]YieldExpression[?In, ?Await]
        ArrowFunction[?In, ?Yield, ?Await]
        AsyncArrowFunction[?In, ?Yield, ?Await]
        LeftHandSideExpression[?Yield, ?Await] = AssignmentExpression[?In, ?Yield, ?Await]
        LeftHandSideExpression[?Yield, ?Await] AssignmentOperator AssignmentExpression[?In, ?Yield, ?Await]
flags: [async]
features: [dynamic-import]
includes: [asyncHelpers.js]
---*/

function tag(arg) {
    return arg[0];
}

async function fn() {
    // MemberExpression TemplateLiteral
    const ns = await import(tag`./module-code-other_FIXTURE.js`); // import('./module-code-other_FIXTURE.js')

    assert.sameValue(ns.local1, 'one six one two');
    assert.sameValue(ns.default, 1612);
}

asyncTest(fn);
