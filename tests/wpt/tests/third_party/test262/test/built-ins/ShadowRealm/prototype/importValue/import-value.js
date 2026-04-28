// Copyright (C) 2021 Rick Waldron. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-shadowrealm.prototype.importvalue
description: >
  ShadowRealm.prototype.importValue can import a value.
flags: [async, module]
features: [ShadowRealm]
---*/

assert.sameValue(
  typeof ShadowRealm.prototype.importValue,
  'function',
  'This test must fail if ShadowRealm.prototype.importValue is not a function'
);

const r = new ShadowRealm();

r.importValue('./import-value_FIXTURE.js', 'x').then(x => {

  assert.sameValue(x, 1);

}).then($DONE, $DONE);
