// Copyright (C) 2018 Leo Balter. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
description: >
    Dynamic Import receives an AssignmentExpression (IdentifierReference: yield)
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
flags: [async, noStrict]
features: [dynamic-import]
includes: [asyncHelpers.js]
---*/

const yield = './module-code_FIXTURE.js';

async function fn() {
    const ns1 = await import(yield); // import('./module-code_FIXTURE.js')

    assert.sameValue(ns1.local1, 'Test262');
    assert.sameValue(ns1.default, 42);
}

asyncTest(fn);
