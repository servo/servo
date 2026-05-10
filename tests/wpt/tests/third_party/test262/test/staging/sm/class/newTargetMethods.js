// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: |
  pending
esid: pending
---*/
// Just like newTargetDirectInvoke, except to prove it works in functions
// defined with method syntax as well. Note that methods, getters, and setters
// are not constructible.

let ol = {
    olTest(arg) { assert.sameValue(arg, 4); assert.sameValue(new.target, undefined); },
    get ol() { assert.sameValue(new.target, undefined); },
    set ol(arg) { assert.sameValue(arg, 4); assert.sameValue(new.target, undefined); }
}

class cl {
    constructor() { assert.sameValue(new.target, cl); }
    clTest(arg) { assert.sameValue(arg, 4); assert.sameValue(new.target, undefined); }
    get cl() { assert.sameValue(new.target, undefined); }
    set cl(arg) { assert.sameValue(arg, 4); assert.sameValue(new.target, undefined); }

    static staticclTest(arg) { assert.sameValue(arg, 4); assert.sameValue(new.target, undefined); }
    static get staticcl() { assert.sameValue(new.target, undefined); }
    static set staticcl(arg) { assert.sameValue(arg, 4); assert.sameValue(new.target, undefined); }
}

const TEST_ITERATIONS = 150;

for (let i = 0; i < TEST_ITERATIONS; i++)
    ol.olTest(4);
for (let i = 0; i < TEST_ITERATIONS; i++)
    ol.ol;
for (let i = 0; i < TEST_ITERATIONS; i++)
    ol.ol = 4;

for (let i = 0; i < TEST_ITERATIONS; i++)
    cl.staticclTest(4);
for (let i = 0; i < TEST_ITERATIONS; i++)
    cl.staticcl;
for (let i = 0; i < TEST_ITERATIONS; i++)
    cl.staticcl = 4;

for (let i = 0; i < TEST_ITERATIONS; i++)
    new cl();

let clInst = new cl();

for (let i = 0; i < TEST_ITERATIONS; i++)
    clInst.clTest(4);
for (let i = 0; i < TEST_ITERATIONS; i++)
    clInst.cl;
for (let i = 0; i < TEST_ITERATIONS; i++)
    clInst.cl = 4;

