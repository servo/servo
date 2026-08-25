// Copyright (C) 2019 Leo Balter. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: The property descriptor WeakRef.prototype
esid: sec-weak-ref.prototype
info: |
  This property has the attributes { [[Writable]]: false, [[Enumerable]]: false,
  [[Configurable]]: false }.
features: [WeakRef]
includes: [propertyHelper.js]
---*/

verifyProperty(WeakRef, 'prototype', {
  writable: false,
  enumerable: false,
  configurable: false
});
