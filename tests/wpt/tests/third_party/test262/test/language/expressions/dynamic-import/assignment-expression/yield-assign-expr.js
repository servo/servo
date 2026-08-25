// Copyright (C) 2018 Leo Balter. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
description: >
    Dynamic Import receives an AssignmentExpression (yield [no LineTerminator here] AssignmentExpression)
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

const a = './module-code_FIXTURE.js';
const b = './module-code-other_FIXTURE.js';

function *g() {
    return import(yield 42);
}

async function fn() {
    let iter = g();
    assert.sameValue(iter.next().value, 42);

    const ns1 = await iter.next(a).value; // import('./module-code_FIXTURE.js')

    assert.sameValue(ns1.local1, 'Test262');
    assert.sameValue(ns1.default, 42);

    iter = g();
    assert.sameValue(iter.next().value, 42);

    const ns2 = await iter.next(b).value; // import('./module-code-other_FIXTURE.js')

    assert.sameValue(ns2.local1, 'one six one two');
    assert.sameValue(ns2.default, 1612);
}

asyncTest(fn);
