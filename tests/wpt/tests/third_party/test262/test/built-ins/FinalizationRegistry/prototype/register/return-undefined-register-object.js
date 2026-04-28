// Copyright (C) 2019 Leo Balter. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-finalization-registry.prototype.register
description: Return undefined after registering an Object
info: |
  FinalizationRegistry.prototype.register ( _target_ , _heldValue_ [, _unregisterToken_ ] )
  1. Let _finalizationRegistry_ be the *this* value.
  2. Perform ? RequireInternalSlot(_finalizationRegistry_, [[Cells]]).
  3. If CanBeHeldWeakly(_target_) is *false*, throw a *TypeError* exception.
  4. If SameValue(_target_, _heldValue_) is *true*, throw a *TypeError*
    exception.
  5. If CanBeHeldWeakly(_unregisterToken_) is *false*,
    a. If _unregisterToken_ is not *undefined*, throw a *TypeError* exception.
    b. Set _unregisterToken_ to ~empty~.
  6. Let _cell_ be the Record { [[WeakRefTarget]]: _target_, [[HeldValue]]:
    _heldValue_, [[UnregisterToken]]: _unregisterToken_ }.
  7. Append _cell_ to _finalizationRegistry_.[[Cells]].
  8. Return *undefined*.
features: [FinalizationRegistry]
---*/

var fn = function() {};
var finalizationRegistry = new FinalizationRegistry(fn);

var target = {};
assert.sameValue(finalizationRegistry.register(target), undefined, 'Register a target');
assert.sameValue(finalizationRegistry.register(target), undefined, 'Register the same target again');
assert.sameValue(finalizationRegistry.register(target), undefined, 'Register the same target again and again');

assert.sameValue(finalizationRegistry.register({}), undefined, 'Register other targets');

assert.sameValue(finalizationRegistry.register(target, undefined, {}), undefined, 'Register target with unregisterToken');
assert.sameValue(
  finalizationRegistry.register(target, undefined, target),
  undefined,
  'Register target with unregisterToken being the registered target'
);

assert.sameValue(finalizationRegistry.register(target, undefined, undefined), undefined, 'Register target with explicit undefined unregisterToken');

assert.sameValue(finalizationRegistry.register(fn), undefined, 'register the cleanup callback fn');
