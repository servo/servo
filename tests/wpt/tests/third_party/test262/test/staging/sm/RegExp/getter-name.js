// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: |
  RegExp getters should have get prefix
info: bugzilla.mozilla.org/show_bug.cgi?id=1180290
esid: pending
---*/

assert.sameValue(Object.getOwnPropertyDescriptor(RegExp, Symbol.species).get.name, "get [Symbol.species]");
assert.sameValue(Object.getOwnPropertyDescriptor(RegExp.prototype, "flags").get.name, "get flags");
assert.sameValue(Object.getOwnPropertyDescriptor(RegExp.prototype, "global").get.name, "get global");
assert.sameValue(Object.getOwnPropertyDescriptor(RegExp.prototype, "ignoreCase").get.name, "get ignoreCase");
assert.sameValue(Object.getOwnPropertyDescriptor(RegExp.prototype, "multiline").get.name, "get multiline");
assert.sameValue(Object.getOwnPropertyDescriptor(RegExp.prototype, "source").get.name, "get source");
assert.sameValue(Object.getOwnPropertyDescriptor(RegExp.prototype, "sticky").get.name, "get sticky");
assert.sameValue(Object.getOwnPropertyDescriptor(RegExp.prototype, "unicode").get.name, "get unicode");
