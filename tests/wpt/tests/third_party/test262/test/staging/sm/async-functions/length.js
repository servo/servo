// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: |
  async function length
info: bugzilla.mozilla.org/show_bug.cgi?id=1185106
esid: pending
---*/

assert.sameValue(async function() {}.length, 0);
assert.sameValue(async function(a) {}.length, 1);
assert.sameValue(async function(a, b, c) {}.length, 3);
assert.sameValue(async function(a, b, c, ...d) {}.length, 3);
