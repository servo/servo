// Copyright (C) 2018 Leo Balter. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
description: >
    Dynamic Import receives an AssignmentExpression (NewTarget)
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
features: [dynamic-import, new.target]
includes: [asyncHelpers.js]
---*/

function ctor() {
    return import(new.target); // import('./module-code_FIXTURE.js')
}

ctor.toString = () => './module-code_FIXTURE.js';

async function fn() {
    const ns = await new ctor();

    assert.sameValue(ns.local1, 'Test262');
    assert.sameValue(ns.default, 42);
}

asyncTest(fn);
