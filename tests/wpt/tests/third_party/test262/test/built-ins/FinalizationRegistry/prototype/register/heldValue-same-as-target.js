// Copyright (C) 2019 Leo Balter. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-finalization-registry.prototype.register
description: heldValue may be the same as target
info: |
  FinalizationRegistry.prototype.register ( _target_ , _heldValue_ [, _unregisterToken_ ] )
  1. Let _finalizationRegistry_ be the *this* value.
  2. Perform ? RequireInternalSlot(_finalizationRegistry_, [[Cells]]).
  3. If CanBeHeldWeakly(_target_) is *false*, throw a *TypeError* exception.
  4. If SameValue(_target_, _heldValue_) is *true*, throw a *TypeError* exception.
features: [FinalizationRegistry, Symbol]
---*/

var finalizationRegistry = new FinalizationRegistry(function() {});

var target = {};
assert.throws(TypeError, () => finalizationRegistry.register(target, target));

// The following will throw regardless of whether the implementation supports
// Symbols as weak values. Step 3 if no, Step 4 if yes.

var symbolTarget = Symbol('a description');
assert.throws(
  TypeError,
  () => finalizationRegistry.register(symbolTarget, symbolTarget),
  'target and heldValue are the same regular symbol'
);

assert.throws(
  TypeError,
  () => finalizationRegistry.register(Symbol.hasInstance, Symbol.hasInstance),
  'target and heldValue are the same well-known symbol'
);
