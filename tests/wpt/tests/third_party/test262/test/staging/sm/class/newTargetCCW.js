// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: |
  pending
esid: pending
---*/
// Make sure we wrap the new target on CCW construct calls.
var g = $262.createRealm().global;

let f = g.eval('(function (expected) { this.accept = new.target === expected; })');

for (let i = 0; i < 1100; i++)
    assert.sameValue(new f(f).accept, true);

