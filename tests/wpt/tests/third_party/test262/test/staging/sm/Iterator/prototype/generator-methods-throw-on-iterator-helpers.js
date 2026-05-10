// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
features:
  - iterator-helpers
description: |
  pending
esid: pending
---*/

const generatorProto = Object.getPrototypeOf(
  Object.getPrototypeOf(
    (function *() {})()
  )
);

const iteratorHelper = [0].values().map(x => x);

assert.throws(TypeError, () => generatorProto.next.call(iteratorHelper));
assert.throws(TypeError, () => generatorProto.return.call(iteratorHelper));
assert.throws(TypeError, () => generatorProto.throw.call(iteratorHelper));

