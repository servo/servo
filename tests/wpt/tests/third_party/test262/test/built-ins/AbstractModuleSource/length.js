// Copyright (C) 2024 Chengzhong Wu. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-properties-of-the-%abstractmodulesource%25-intrinsic-object
description: >
  %AbstractModuleSource%.length property descriptor
info: |
  28.1.1.1 %AbstractModuleSource% ( )

includes: [propertyHelper.js]
features: [source-phase-imports]
flags: [module]
---*/

assert.sameValue(typeof $262.AbstractModuleSource, 'function');
verifyProperty($262.AbstractModuleSource, 'length', {
  value: 0,
  writable: false,
  enumerable: false,
  configurable: true
});
