// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: |
  pending
esid: pending
---*/
var obj = { get a() { return 1; } };
assert.throws(TypeError, () => {
    new Object.getOwnPropertyDescriptor(obj, "a").get
});

obj = { set a(b) { } };
assert.throws(TypeError, () => {
    new Object.getOwnPropertyDescriptor(obj, "a").set
});

obj = { get a() { return 1; }, set a(b) { } };
assert.throws(TypeError, () => {
    new Object.getOwnPropertyDescriptor(obj, "a").get
});
assert.throws(TypeError, () => {
    new Object.getOwnPropertyDescriptor(obj, "a").set
});

