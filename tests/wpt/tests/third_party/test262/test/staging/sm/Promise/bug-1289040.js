// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: |
  pending
esid: pending
---*/

var global = $262.createRealm().global;
Promise.prototype.then = global.Promise.prototype.then;
var p1 = new Promise(function f(r) {
    r(1);
});
var p2 = p1.then(function g(){});
