// Copyright (C) 2021 Rick Waldron. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-shadowrealm.prototype.importvalue
description: >
  ShadowRealm.prototype.importValue is not a constructor.
includes: [isConstructor.js]
features: [ShadowRealm, Reflect.construct]
---*/

assert.sameValue(
  typeof ShadowRealm.prototype.importValue,
  'function',
  'This test must fail if ShadowRealm.prototype.importValue is not a function'
);

assert.sameValue(
  isConstructor(ShadowRealm.prototype.importValue),
  false,
  'isConstructor(ShadowRealm.prototype.importValue) must return false'
);

assert.throws(TypeError, () => {
  new ShadowRealm.prototype.importValue("", "name");
}, '`new ShadowRealm.prototype.importValue("")` throws TypeError');

const r = new ShadowRealm();

assert.throws(TypeError, () => {
  new r.importValue("./import-value_FIXTURE.js", "x");
}, '`new r.importValue("...")` throws TypeError');
