// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: |
  pending
esid: pending
---*/

var getProtoCalled = false;

var newTarget = Object.defineProperty(function(){}.bind(), "prototype", {
    get() {
        getProtoCalled = true;
        return null;
    }
});

var AsyncGenerator = async function*(){}.constructor;

assert.throws(SyntaxError, () => {
    Reflect.construct(AsyncGenerator, ["@error"], newTarget);
});

assert.sameValue(getProtoCalled, false);
