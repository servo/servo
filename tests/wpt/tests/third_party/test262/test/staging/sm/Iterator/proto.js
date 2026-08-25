// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
  The prototype of the Iterator constructor is the intrinsic object %FunctionPrototype%.

  Iterator is not enabled unconditionally
features:
  - iterator-helpers
description: |
  pending
esid: pending
---*/
assert.sameValue(Object.getPrototypeOf(Iterator), Function.prototype);

const propDesc = Reflect.getOwnPropertyDescriptor(Iterator, 'prototype');
assert.sameValue(propDesc.writable, false);
assert.sameValue(propDesc.enumerable, false);
assert.sameValue(propDesc.configurable, false);

