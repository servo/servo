// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-weak-ref-target
description: >
  Returns a new ordinary object from the WeakRef constructor when called with a
  Symbol target
info: |
  WeakRef ( _target_ )
  3. Let _weakRef_ be ? OrdinaryCreateFromConstructor(NewTarget,
    *"%WeakRefPrototype%"*, « [[WeakRefTarget]] »).
  4. Perfom AddToKeptObjects(_target_).
  5. Set _weakRef_.[[WeakRefTarget]] to _target_.
  6. Return _weakRef_.
features: [Symbol, WeakRef, symbols-as-weakmap-keys]
---*/

var target = Symbol('a description');
var wr = new WeakRef(target);

assert.sameValue(wr instanceof WeakRef, true, 'object is instanceof WeakRef');
assert.sameValue(Object.getPrototypeOf(wr), WeakRef.prototype, 'prototype is WeakRef.prototype');

wr = new WeakRef(Symbol.hasInstance);

assert.sameValue(wr instanceof WeakRef, true, 'object is instanceof WeakRef');
assert.sameValue(Object.getPrototypeOf(wr), WeakRef.prototype, 'prototype is WeakRef.prototype');

