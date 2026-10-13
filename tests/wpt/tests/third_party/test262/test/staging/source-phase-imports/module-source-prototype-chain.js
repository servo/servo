// Copyright (C) 2025 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
description: >
  The [[ModuleSource]] object's [[Prototype]]'s [[Prototype]] must be
  %AbstractModuleSource%.prototype.
esid: sec-abstract-module-records
info: |
  Table 3: Module Record Fields

  [[ModuleSource]]
  The Module Source Object corresponding to this source Module Record's
  source phase, or ~empty~ if it is not available for this module kind.
  When not ~empty~, it must be an object whose initial [[Prototype]] is
  an object whose initial [[Prototype]] is %AbstractModuleSource%.prototype.

features: [source-phase-imports, source-phase-imports-module-source]
flags: [async]
includes: [asyncHelpers.js]
---*/

asyncTest(async function () {
  const moduleSource = await import.source('<module source>');

  assert.sameValue(typeof moduleSource, 'object',
    'import.source() must resolve to an object');

  const proto = Object.getPrototypeOf(moduleSource);
  assert.notSameValue(proto, null,
    'The [[ModuleSource]] object must have a [[Prototype]]');

  const protoProto = Object.getPrototypeOf(proto);
  assert.sameValue(protoProto, $262.AbstractModuleSource.prototype,
    'The [[Prototype]] of the [[ModuleSource]] object\'s [[Prototype]] must be %AbstractModuleSource%.prototype');
});
