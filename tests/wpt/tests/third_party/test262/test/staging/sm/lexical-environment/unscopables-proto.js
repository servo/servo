// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
flags:
  - noStrict
description: |
  pending
esid: pending
---*/
// @@unscopables treats properties found on prototype chains the same as other
// properties.

const x = "global x";
const y = "global y";

// obj[@@unscopables].x works when obj.x is inherited via the prototype chain.
let proto = {x: "object x", y: "object y"};
let env = Object.create(proto);
env[Symbol.unscopables] = {x: true, y: false};
with (env) {
    assert.sameValue(x, "global x");
    assert.sameValue(delete x, false);
    assert.sameValue(y, "object y");
}
assert.sameValue(env.x, "object x");

// @@unscopables works if is inherited via the prototype chain.
env = {
    x: "object",
    [Symbol.unscopables]: {x: true, y: true}
};
for (let i = 0; i < 50; i++)
    env = Object.create(env);
env.y = 1;
with (env) {
    assert.sameValue(x, "global x");
    assert.sameValue(y, "global y");
}

// @@unscopables works if the obj[@@unscopables][id] property is inherited.
env = {
    x: "object",
    [Symbol.unscopables]: Object.create({x: true})
};
with (env)
    assert.sameValue(x, "global x");

