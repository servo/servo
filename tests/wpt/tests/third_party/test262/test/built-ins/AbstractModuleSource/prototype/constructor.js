// Copyright (C) 2024 Chengzhong Wu. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-properties-of-the-%abstractmodulesource%25-prototype-object
description: >
  %AbstractModuleSource%.prototype.constructor property descriptor
info: |
  %AbstractModuleSource%.prototype.constructor

  The initial value of %AbstractModuleSource%.prototype.constructor is %AbstractModuleSource%.

includes: [propertyHelper.js]
features: [source-phase-imports]
flags: [module]
---*/

assert.sameValue(typeof $262.AbstractModuleSource, 'function');
verifyProperty($262.AbstractModuleSource.prototype, 'constructor', {
  value: $262.AbstractModuleSource,
  writable: true,
  enumerable: false,
  configurable: true
});
