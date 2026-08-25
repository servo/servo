// Copyright (C) 2019 Leo Balter. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-finalization-registry.prototype.unregister
description: >
  Return boolean values indicating unregistering of values with Object token
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
features: [FinalizationRegistry]
---*/

var fn = function() {};
var finalizationRegistry = new FinalizationRegistry(fn);

var target1 = {};
var target2 = {};
var target3 = {};
var token1 = {};
var token2 = {};

assert.sameValue(finalizationRegistry.unregister(token1), false, 'unregistering token1 from empty finalizationRegistry');
assert.sameValue(finalizationRegistry.unregister(token2), false, 'unregistering token2 from empty finalizationRegistry');

finalizationRegistry.register(target1, undefined, token1);
finalizationRegistry.register(target1, undefined, token1); // Repeat registering un purpose
finalizationRegistry.register(target2, undefined, token2);
finalizationRegistry.register(target3, undefined, token2);

assert.sameValue(finalizationRegistry.unregister(target1), false, 'own target does not work on unregister, #1');
assert.sameValue(finalizationRegistry.unregister(target2), false, 'own target does not work on unregister, #2');
assert.sameValue(finalizationRegistry.unregister(target3), false, 'own target does not work on unregister, #3');

assert.sameValue(finalizationRegistry.unregister(token1), true, 'unregistering token1 from finalizationRegistry');
assert.sameValue(finalizationRegistry.unregister(token1), false, 'unregistering token1 again from finalizationRegistry');
assert.sameValue(finalizationRegistry.unregister(token2), true, 'unregistering token2 to remove target2 and target3');
assert.sameValue(finalizationRegistry.unregister(token2), false, 'unregistering token2 from empty finalizationRegistry');

// Notice these assertions take advantage of adding targets previously added with a token,
// but now they got no token so it won't be used to remove them.
finalizationRegistry.register(target1, token1); // holdings, not unregisterToken
finalizationRegistry.register(target2, token2); // holdings, not unregisterToken
finalizationRegistry.register(target3);

assert.sameValue(finalizationRegistry.unregister(token1), false, 'nothing to remove without a set unregisterToken #1');
assert.sameValue(finalizationRegistry.unregister(token2), false, 'nothing to remove without a set unregisterToken #2');
