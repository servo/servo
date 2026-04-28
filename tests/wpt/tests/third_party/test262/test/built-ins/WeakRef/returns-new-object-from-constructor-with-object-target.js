// Copyright (C) 2019 Leo Balter. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-weak-ref-target
description: >
  Returns a new ordinary object from the WeakRef constructor when called with an
  Object target
info: |
  WeakRef ( _target_ )
  3. Let _weakRef_ be ? OrdinaryCreateFromConstructor(NewTarget,
    *"%WeakRefPrototype%"*, « [[WeakRefTarget]] »).
  4. Perfom AddToKeptObjects(_target_).
  5. Set _weakRef_.[[WeakRefTarget]] to _target_.
  6. Return _weakRef_.
features: [WeakRef]
---*/

var target = {};
var wr = new WeakRef(target);

assert.notSameValue(wr, target, 'does not return the same object');
assert.sameValue(wr instanceof WeakRef, true, 'instanceof');

for (let key of Object.getOwnPropertyNames(wr)) {
  assert(false, `should not set any own named properties: ${key}`);
}

for (let key of Object.getOwnPropertySymbols(wr)) {
  assert(false, `should not set any own symbol properties: ${String(key)}`);
}

assert.sameValue(Object.getPrototypeOf(wr), WeakRef.prototype);


