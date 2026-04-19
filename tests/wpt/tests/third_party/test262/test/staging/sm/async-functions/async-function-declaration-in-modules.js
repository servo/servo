// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
flags:
  - module
description: |
  pending
esid: pending
---*/

async function f() {
    return "success";
}

var AsyncFunction = (async function(){}).constructor;

assert.sameValue(f instanceof AsyncFunction, true);

f().then(v => {
    assert.sameValue("success", v);
});
