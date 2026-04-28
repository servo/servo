// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: |
  pending
esid: pending
---*/
function g1() {
    throw 10;
}

function g2() {
    throw 20;
}

class A {
    #x = "hello" + g1();
    constructor(o = g2()) {
    }
};

var thrown;
try {
    new A;
} catch (e) {
    thrown = e;
}

assert.sameValue(thrown, 10);

