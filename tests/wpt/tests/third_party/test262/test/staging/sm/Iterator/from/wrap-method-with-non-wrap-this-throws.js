// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
features:
  - iterator-helpers
info: |
  Iterator is not enabled unconditionally
description: |
  pending
esid: pending
---*/
// All methods on %WrapForValidIteratorPrototype% require an [[Iterated]]
// internal slot on the `this` object.

class TestIterator {
  next() {
    return {
      done: false,
      value: 0,
    };
  }
}

const nextMethod = Iterator.from(new TestIterator()).next;
assert.throws(TypeError, () => nextMethod.call(undefined));
assert.throws(TypeError, () => nextMethod.call(null));
assert.throws(TypeError, () => nextMethod.call(0));
assert.throws(TypeError, () => nextMethod.call(false));
assert.throws(TypeError, () => nextMethod.call('test'));
assert.throws(TypeError, () => nextMethod.call(Object(1)));
assert.throws(TypeError, () => nextMethod.call({}));

const returnMethod = Iterator.from(new TestIterator()).next;
assert.throws(TypeError, () => returnMethod.call(undefined));
assert.throws(TypeError, () => returnMethod.call(null));
assert.throws(TypeError, () => returnMethod.call(0));
assert.throws(TypeError, () => returnMethod.call(false));
assert.throws(TypeError, () => returnMethod.call('test'));
assert.throws(TypeError, () => returnMethod.call(Object(1)));
assert.throws(TypeError, () => returnMethod.call({}));

const throwMethod = Iterator.from(new TestIterator()).next;
assert.throws(TypeError, () => throwMethod.call(undefined));
assert.throws(TypeError, () => throwMethod.call(null));
assert.throws(TypeError, () => throwMethod.call(0));
assert.throws(TypeError, () => throwMethod.call(false));
assert.throws(TypeError, () => throwMethod.call('test'));
assert.throws(TypeError, () => throwMethod.call(Object(1)));
assert.throws(TypeError, () => throwMethod.call({}));

