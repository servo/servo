// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
flags:
  - noStrict
description: |
  pending
esid: pending
---*/
// @@unscopables checks can call getters.

// The @@unscopables property itself can be a getter.
let hit1 = 0;
let x = "global x";
let env1 = {
    x: "env1.x",
    get [Symbol.unscopables]() {
        hit1++;
        return {x: true};
    }
};
with (env1)
    assert.sameValue(x, "global x");
assert.sameValue(hit1, 1);

// It can throw; the exception is propagated out.
function Fit() {}
with ({x: 0, get [Symbol.unscopables]() { throw new Fit; }})
    assert.throws(Fit, () => x);

// Individual properties on the @@unscopables object can have getters.
let hit2 = 0;
let env2 = {
    x: "env2.x",
    [Symbol.unscopables]: {
        get x() {
            hit2++;
            return true;
        }
    }
};
with (env2)
    assert.sameValue(x, "global x");
assert.sameValue(hit2, 1);

// And they can throw.
with ({x: 0, [Symbol.unscopables]: {get x() { throw new Fit; }}})
    assert.throws(Fit, () => x);

