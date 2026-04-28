// Copyright (C) 2019 Leo Balter. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: The property descriptor FinalizationRegistry.prototype
esid: sec-finalization-registry.prototype
info: |
  This property has the attributes { [[Writable]]: false, [[Enumerable]]: false,
  [[Configurable]]: false }.
features: [FinalizationRegistry]
includes: [propertyHelper.js]
---*/

verifyProperty(FinalizationRegistry, 'prototype', {
  writable: false,
  enumerable: false,
  configurable: false
});
