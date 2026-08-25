// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-finalization-registry.prototype.unregister
description: >
  Return boolean values indicating unregistering of values with Symbol token
info: |
  FinalizationRegistry.prototype.unregister ( _unregisterToken_ )
  4. Let _removed_ be *false*.
  5. For each Record { [[WeakRefTarget]], [[HeldValue]], [[UnregisterToken]] }
    _cell_ of _finalizationRegistry_.[[Cells]], do
    a. If _cell_.[[UnregisterToken]] is not ~empty~ and
      SameValue(_cell_.[[UnregisterToken]], _unregisterToken_) is *true*, then
      i. Remove _cell_ from _finalizationRegistry_.[[Cells]].
      ii. Set _removed_ to *true*.
  6. Return _removed_.
features: [FinalizationRegistry, Symbol, symbols-as-weakmap-keys]
---*/

var fn = function() {};
var reg = new FinalizationRegistry(fn);

var target1 = {};
var target2 = {};
var target3 = {};
var token = Symbol('unregister');

assert.sameValue(reg.unregister(token), false, 'unregistering regular symbol from empty registry');
assert.sameValue(reg.unregister(Symbol.hasInstance), false, 'unregistering well-known symbol from empty registry');

reg.register(target1, undefined, token);
reg.register(target1, undefined, token); // Repeat registering on purpose
reg.register(target2, undefined, Symbol.hasInstance);
reg.register(target3, undefined, Symbol.hasInstance);

assert.sameValue(reg.unregister(token), true, 'unregistering regular symbol from finalization registry');
assert.sameValue(reg.unregister(token), false, 'unregistering regular symbol again from finalization registry');
assert.sameValue(
  reg.unregister(Symbol.hasInstance),
  true,
  'unregistering well-known symbol to remove target2 and target3'
);
assert.sameValue(
  reg.unregister(Symbol.hasInstance),
  false,
  'unregistering well-known again from finalization registry'
);

// Notice these assertions take advantage of adding targets previously added
// with a token, but now they have no token so it won't be used to remove them.
reg.register(target1, token); // heldValue, not unregisterToken
reg.register(target2, Symbol.hasInstance); // heldValue, not unregisterToken
reg.register(target3);

assert.sameValue(reg.unregister(token), false, 'nothing to remove with regular symbol unregister token');
assert.sameValue(
  reg.unregister(Symbol.hasInstance),
  false,
  'nothing to remove with well-known symbol unregister token'
);
