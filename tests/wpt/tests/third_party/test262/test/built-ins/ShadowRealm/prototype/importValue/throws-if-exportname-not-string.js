// Copyright (C) 2021 Rick Waldron. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-shadowrealm.prototype.importvalue
description: >
  ShadowRealm.prototype.importValue throws if exportName is not a string.
features: [ShadowRealm]
---*/

assert.sameValue(
  typeof ShadowRealm.prototype.importValue,
  'function',
  'This test must fail if ShadowRealm.prototype.importValue is not a function'
);

const r = new ShadowRealm();
let count = 0;

const exportName = {
  toString() {
    count += 1;
    throw new Test262Error();
  }
};

assert.throws(TypeError, () => {
  r.importValue('', exportName);
});

assert.sameValue(count, 0);
