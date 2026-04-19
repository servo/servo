// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
includes: [compareArray.js]
description: |
  RegExp.prototype.split should reflect the change to Object.prototype.flags.
info: bugzilla.mozilla.org/show_bug.cgi?id=0
esid: pending
---*/

Object.defineProperty(Object.prototype, "flags", Object.getOwnPropertyDescriptor(RegExp.prototype, "flags"));
delete RegExp.prototype.flags;

let re = /a/i;
let a = re[Symbol.split]("1a2A3a4A5");
assert.compareArray(a, ["1", "2", "3", "4", "5"]);

delete Object.prototype.flags;

Object.prototype.flags = "";

a = re[Symbol.split]("1a2A3a4A5");
assert.compareArray(a, ["1", "2A3", "4A5"]);
