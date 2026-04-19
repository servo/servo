// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: |
  DataView getters should have get prefix
info: bugzilla.mozilla.org/show_bug.cgi?id=1180290
esid: pending
---*/

assert.sameValue(Object.getOwnPropertyDescriptor(DataView.prototype, "buffer").get.name, "get buffer");
assert.sameValue(Object.getOwnPropertyDescriptor(DataView.prototype, "byteLength").get.name, "get byteLength");
assert.sameValue(Object.getOwnPropertyDescriptor(DataView.prototype, "byteOffset").get.name, "get byteOffset");
