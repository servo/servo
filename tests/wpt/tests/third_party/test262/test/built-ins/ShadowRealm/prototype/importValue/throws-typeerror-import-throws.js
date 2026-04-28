// Copyright (C) 2021 Chengzhong Wu. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-realmimportvalue
description: >
  ShadowRealm.prototype.importValue rejects with TypeError when the imported script throws.
info: |
  RealmImportValue ( specifierString, exportNameString, callerRealm, evalRealm, evalContext )

    ...
    17. Return ! PerformPromiseThen(innerCapability.[[Promise]], onFulfilled, callerRealm.[[Intrinsics]].[[%ThrowTypeError%]], promiseCapability).

flags: [async, module]
features: [ShadowRealm]
---*/

assert.sameValue(
  typeof ShadowRealm.prototype.importValue,
  'function',
  'This test must fail if ShadowRealm.prototype.importValue is not a function'
);

const r = new ShadowRealm();

r.importValue('./import-value_throws_FIXTURE.js', 'y')
  .then(
    () => {
      throw new Test262Error("unreachable");
    },
    err => {
      assert.sameValue(Object.getPrototypeOf(err), TypeError.prototype, 'should be rejected with TypeError');
    }
  )
  .then($DONE, $DONE);
