// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: |
  pending
esid: pending
---*/
class foo extends Array { }

function testArrs(arrs) {
    for (let arr of arrs) {
        assert.sameValue(Object.getPrototypeOf(arr), foo.prototype);
    }
}

var arrs = [];
for (var i = 0; i < 25; i++)
    arrs.push(new foo(1));

testArrs(arrs);

arrs[0].nonIndexedProp = "uhoh";

arrs.push(new foo(1));

testArrs(arrs);

