// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: |
  pending
esid: pending
---*/

// Super property accesses should play nice with the pretty printer.
class testNonExistent {
    constructor() {
        super["prop"]();
    }
}

// Should fold to super.prop
assert.throws(TypeError, () => new testNonExistent());

var ol = { testNonExistent() { super.prop(); } };
assert.throws(TypeError, () => ol.testNonExistent());

var olElem = { testNonExistent() { var prop = "prop"; super[prop](); } };
assert.throws(TypeError, () => olElem.testNonExistent());
