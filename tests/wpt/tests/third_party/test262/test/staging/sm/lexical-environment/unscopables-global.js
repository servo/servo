// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
flags:
  - noStrict
description: |
  pending
esid: pending
---*/
// @@unscopables does not affect the global environment.

this.x = "global property x";
let y = "global lexical y";
this[Symbol.unscopables] = {x: true, y: true};
assert.sameValue(x, "global property x");
assert.sameValue(y, "global lexical y");
assert.sameValue(eval("x"), "global property x");
assert.sameValue(eval("y"), "global lexical y");

// But it does affect `with` statements targeting the global object.
{
    let x = "local x";
    with (this)
        assert.sameValue(x, "local x");
}

