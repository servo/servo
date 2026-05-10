// Copyright (C) 2024 Chengzhong Wu. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-properties-of-the-%abstractmodulesource%25-intrinsic-object
description: >
  The prototype of %AbstractModuleSource% is Object.prototype
info: |
  The value of the [[Prototype]] internal slot of the %AbstractModuleSource% object is the
  intrinsic object %FunctionPrototype%.
features: [source-phase-imports]
flags: [module]
---*/

assert.sameValue(typeof $262.AbstractModuleSource, 'function');
assert.sameValue(
  Object.getPrototypeOf($262.AbstractModuleSource),
  Function.prototype,
  'Object.getPrototypeOf(AbstractModuleSource) returns the value of `Function.prototype`'
);
