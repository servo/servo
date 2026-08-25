// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: >
    Objects whose specified property is not writable satisfy the assertion.
includes: [propertyHelper.js]
---*/

var obj = {};

Object.defineProperty(obj, 'a', {
  writable: false,
  value: 123
});

verifyNotWritable(obj, 'a');

if (obj.a !== 123) {
  throw new Error('`verifyNotWritable` should be non-destructive.');
}
