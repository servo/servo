// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
flags:
  - noStrict
description: |
  pending
esid: pending
---*/
// If obj[@@unscopables][id], then `delete id` works across `with (obj)` scope.

this.niche = 7;
let obj = { niche: 8, [Symbol.unscopables]: { niche: true } };
with (obj) {
    delete niche;
}

assert.sameValue(obj.niche, 8);
assert.sameValue("niche" in this, false);

// Same thing, but delete a variable introduced by sloppy direct eval.
this.niche = 9;
function f() {
    eval("var niche = 10;");
    with (obj) {
        assert.sameValue(niche, 10);
        delete niche;
    }
    assert.sameValue(niche, 9);
}

// Of course none of this affects a qualified delete.
assert.sameValue(delete this.niche, true);
assert.sameValue("niche" in this, false);

