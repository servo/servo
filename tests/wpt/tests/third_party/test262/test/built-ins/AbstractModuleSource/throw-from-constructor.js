// Copyright (C) 2024 Chengzhong Wu. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-%abstractmodulesource%25-constructor
description: The %AbstractModuleSource% constructor will throw an error when invoked
info: |
  28.1.1.1 %AbstractModuleSource% ( )
  This function performs the following steps when called:

  1. Throw a TypeError exception.
features: [source-phase-imports]
flags: [module]
---*/

assert.sameValue(typeof $262.AbstractModuleSource, 'function');
assert.throws(TypeError, function() {
  new $262.AbstractModuleSource();
}, '%AbstractModuleSource%() should throw TypeError');
