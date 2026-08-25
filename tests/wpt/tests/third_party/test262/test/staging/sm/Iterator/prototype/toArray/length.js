// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: |
  The `length` property of Iterator.prototype.toArray.
info: |
  ES7 section 17: Unless otherwise specified, the length property of a built-in
  Function object has the attributes { [[Writable]]: false, [[Enumerable]]:
  false, [[Configurable]]: true }.

  Iterator is not enabled unconditionally
features:
  - iterator-helpers
esid: pending
---*/
const propDesc = Reflect.getOwnPropertyDescriptor(Iterator.prototype.toArray, 'length');
assert.sameValue(propDesc.value, 0);
assert.sameValue(propDesc.writable, false);
assert.sameValue(propDesc.enumerable, false);
assert.sameValue(propDesc.configurable, true);

