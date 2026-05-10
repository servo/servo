// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
flags:
  - noStrict
description: |
  pending
esid: pending
---*/
// @@unscopables continues to work after exiting the relevant `with` block,
// if the environment is captured by a closure.

let env = {
    x: 9000,
    [Symbol.unscopables]: {x: true}
};

function make_adder(x) {
    with (env)
        return function (y) { return x + y; };
}
assert.sameValue(make_adder(3)(10), 13);

// Same test, but with a bunch of different parts for bad luck
let x = 500;
function make_adder_with_eval() {
    with (env)
        return eval('y => eval("x + y")');
}
assert.sameValue(make_adder_with_eval()(10), 510);

