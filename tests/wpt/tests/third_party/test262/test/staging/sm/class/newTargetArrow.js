// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: |
  pending
esid: pending
---*/
// new.target is valid in any arrow function not in a global context.
new Function('(() => new.target)()');

// It's also good inside eval, but not global eval
assert.throws(SyntaxError, () => eval('() => new.target'));

function assertNewTarget(expected) {
    assert.sameValue((()=>new.target)(), expected);
    assert.sameValue(eval('()=>new.target')(), expected);

    // Make sure that arrow functions can escape their original context and
    // still get the right answer.
    return (() => new.target);
}

const ITERATIONS = 550;
for (let i = 0; i < ITERATIONS; i++)
    assert.sameValue(assertNewTarget(undefined)(), undefined);

for (let i = 0; i < ITERATIONS; i++)
    assert.sameValue(new assertNewTarget(assertNewTarget)(), assertNewTarget);

