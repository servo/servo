// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: pending
description: |
  %Iterator.prototype%.drop.name value and descriptor.
info: |
  17 ECMAScript Standard Built-in Objects
features:
  - iterator-helpers
---*/
assert.sameValue(Iterator.prototype.drop.name, 'drop');

const propertyDescriptor = Reflect.getOwnPropertyDescriptor(Iterator.prototype.drop, 'name');
assert.sameValue(propertyDescriptor.value, 'drop');
assert.sameValue(propertyDescriptor.enumerable, false);
assert.sameValue(propertyDescriptor.writable, false);
assert.sameValue(propertyDescriptor.configurable, true);

