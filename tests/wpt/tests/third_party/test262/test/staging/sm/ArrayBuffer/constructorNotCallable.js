// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: |
  pending
esid: pending
---*/

assert.throws(TypeError, () => ArrayBuffer());
assert.throws(TypeError, () => ArrayBuffer(1));
assert.throws(TypeError, () => ArrayBuffer.call(null));
assert.throws(TypeError, () => ArrayBuffer.apply(null, []));
assert.throws(TypeError, () => Reflect.apply(ArrayBuffer, null, []));
