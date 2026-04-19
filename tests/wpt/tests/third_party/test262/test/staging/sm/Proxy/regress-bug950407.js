// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: |
  pending
esid: pending
---*/
var ab = new ArrayBuffer(5);
var p = new Proxy(ab, {});
var ps = Object.getOwnPropertyDescriptor(Object.prototype, "__proto__").set;
var new_proto = {};
ps.call(p, new_proto);

assert.sameValue(ab.__proto__, new_proto);
