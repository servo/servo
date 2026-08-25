// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: |
  pending
esid: pending
---*/
for (let method of [undefined, null]) {
    let obj = {
        [Symbol.toPrimitive]: method,
        toString: () => "pass",
    };
    assert.sameValue("" + obj, "pass");
}

for (let method of [true, false, 0, 123, "", "abc", Symbol(), {}]) {
    let obj = {
        [Symbol.toPrimitive]: method,
        toString: () => "pass",
    };
    assert.throws(TypeError, () => "" + obj);
}

