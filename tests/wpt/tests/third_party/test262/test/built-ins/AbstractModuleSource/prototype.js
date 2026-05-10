// Copyright (C) 2024 Chengzhong Wu. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-%abstractmodulesource%25.prototype
description: >
  %AbstractModuleSource%.prototype property descriptor
info: |
  28.3.2.1 %AbstractModuleSource%.prototype

  The initial value of %AbstractModuleSource%.prototype is the %AbstractModuleSource% prototype object.
  This property has the attributes { [[Writable]]: false, [[Enumerable]]: false, [[Configurable]]: false }.
includes: [propertyHelper.js]
features: [source-phase-imports]
flags: [module]
---*/

assert.sameValue(typeof $262.AbstractModuleSource, 'function');
verifyProperty($262.AbstractModuleSource, 'prototype', {
  value: $262.AbstractModuleSource.prototype,
  writable: false,
  enumerable: false,
  configurable: false
});
