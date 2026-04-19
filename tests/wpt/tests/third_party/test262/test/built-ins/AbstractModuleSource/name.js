// Copyright (C) 2024 Chengzhong Wu. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-properties-of-the-%abstractmodulesource%25-intrinsic-object
description: >
  %AbstractModuleSource%.name property descriptor
info: |
  The %AbstractModuleSource% intrinsic object has a "name" property whose value is "AbstractModuleSource".

  Unless otherwise specified, the name property of a built-in function
  object, if it exists, has the attributes { [[Writable]]: false,
  [[Enumerable]]: false, [[Configurable]]: true }.
includes: [propertyHelper.js]
features: [source-phase-imports]
flags: [module]
---*/

assert.sameValue(typeof $262.AbstractModuleSource, 'function');
verifyProperty($262.AbstractModuleSource, 'name', {
  value: 'AbstractModuleSource',
  writable: false,
  enumerable: false,
  configurable: true
});
