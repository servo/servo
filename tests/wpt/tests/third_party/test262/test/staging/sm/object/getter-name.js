// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: |
  Object accessors should have get prefix
info: bugzilla.mozilla.org/show_bug.cgi?id=1180290
esid: pending
---*/

assert.sameValue(Object.getOwnPropertyDescriptor(Object.prototype, "__proto__").get.name, "get __proto__");
assert.sameValue(Object.getOwnPropertyDescriptor(Object.prototype, "__proto__").set.name, "set __proto__");
