// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
flags:
  - noStrict
description: |
  pending
esid: pending
---*/
// When env[@@unscopables].x changes, bindings can appear even if env is inextensible.

let x = "global";
let unscopables = {x: true};
let env = Object.create(null);
env[Symbol.unscopables] = unscopables;
env.x = "object";
Object.freeze(env);

for (let i = 0; i < 1004; i++) {
    if (i === 1000)
        unscopables.x = false;
    with (env) {
        assert.sameValue(x, i < 1000 ? "global" : "object");
    }
}

