// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: pending
description: |
  %Iterator.prototype%.filter.name value and descriptor.
info: |
  17 ECMAScript Standard Built-in Objects
features:
  - iterator-helpers
---*/
assert.sameValue(Iterator.prototype.filter.name, 'filter');

const propertyDescriptor = Reflect.getOwnPropertyDescriptor(Iterator.prototype.filter, 'name');
assert.sameValue(propertyDescriptor.value, 'filter');
assert.sameValue(propertyDescriptor.enumerable, false);
assert.sameValue(propertyDescriptor.writable, false);
assert.sameValue(propertyDescriptor.configurable, true);

