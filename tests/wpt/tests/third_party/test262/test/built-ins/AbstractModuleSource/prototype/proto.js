// Copyright (C) 2024 Chengzhong Wu. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-properties-of-the-%abstractmodulesource%25-prototype-object
description: The prototype of %AbstractModuleSource%.prototype is Object.prototype
info: |
  The %AbstractModuleSource% prototype object has a [[Prototype]] internal slot whose value is %Object.prototype%.

features: [source-phase-imports]
flags: [module]
---*/

assert.sameValue(typeof $262.AbstractModuleSource, 'function');
var proto = Object.getPrototypeOf($262.AbstractModuleSource.prototype);
assert.sameValue(proto, Object.prototype);
