// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
features:
  - iterator-helpers
description: |
  pending
esid: pending
---*/

const iteratorHelperProto = Object.getPrototypeOf([].values().map(x => x));

function *gen() {
  yield 1;
}

assert.throws(TypeError, () => iteratorHelperProto.next.call(gen()));
assert.throws(TypeError, () => iteratorHelperProto.return.call(gen()));
assert.throws(TypeError, () => iteratorHelperProto.throw.call(gen()));

