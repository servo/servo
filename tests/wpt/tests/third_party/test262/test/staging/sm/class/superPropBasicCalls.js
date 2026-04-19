// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: |
  pending
esid: pending
---*/
// Super property (and calls) works in non-extending classes and object
// litterals.
class toStringTest {
    constructor() {
        // Install a property to make it plausible that it's the same this
        this.foo = "rhinoceros";
    }

    test() {
        assert.sameValue(super.toString(), super["toString"]());
        assert.sameValue(super.toString(), this.toString());
    }
}

new toStringTest().test();

let toStrOL = {
    test() {
        assert.sameValue(super.toString(), super["toString"]());
        assert.sameValue(super.toString(), this.toString());
    }
}

toStrOL.test();

