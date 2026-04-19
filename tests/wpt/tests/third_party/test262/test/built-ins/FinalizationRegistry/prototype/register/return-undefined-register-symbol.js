// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-finalization-registry.prototype.register
description: Return undefined after registering a Symbol
info: |
  FinalizationRegistry.prototype.register ( _target_ , _heldValue_ [, _unregisterToken_ ] )
  8. Return *undefined*.
features: [FinalizationRegistry, Symbol, symbols-as-weakmap-keys]
---*/

var fn = function() {};
var reg = new FinalizationRegistry(fn);

var target = Symbol('a description');
assert.sameValue(reg.register(target), undefined, 'Register a regular symbol');
assert.sameValue(reg.register(target), undefined, 'Register the same symbol again');
assert.sameValue(reg.register(target), undefined, 'Register the same symbol a third time');

assert.sameValue(
  reg.register(Symbol('a description')),
  undefined,
  'Register another symbol with the same description'
);
assert.sameValue(
  reg.register(Symbol('a different description')),
  undefined,
  'Register another symbol with another description'
);

assert.sameValue(
  reg.register(target, undefined, Symbol('unregister token')),
  undefined,
  'Register a regular symbol with a symbol unregister token'
);
assert.sameValue(
  reg.register(target, undefined, target),
  undefined,
  'Register a regular symbol with itself as the unregister token'
);

assert.sameValue(
  reg.register(target, undefined, undefined),
  undefined,
  'Register a regular symbol with explicit undefined unregister token'
);

assert.sameValue(reg.register(Symbol.hasInstance), undefined, 'Register a well-known symbol');
assert.sameValue(reg.register(Symbol.hasInstance), undefined, 'Register the same well-known symbol again');
assert.sameValue(reg.register(Symbol.hasInstance), undefined, 'Register the same well-known symbol a third time');

assert.sameValue(
  reg.register(target, undefined, Symbol.hasInstance),
  undefined,
  'Register a regular symbol with a well-known symbol unregister token'
);
assert.sameValue(
  reg.register(Symbol.hasInstance, undefined, Symbol.iterator),
  undefined,
  'Register a well-known symbol with a different well-known symbol as unregister token'
);
assert.sameValue(
  reg.register(Symbol.hasInstance, undefined, Symbol.hasInstance),
  undefined,
  'Register a well-known symbol with itself as the unregister token'
);

assert.sameValue(
  reg.register(Symbol.hasInstance, undefined, undefined),
  undefined,
  'Register a well-known symbol with explicit undefined unregister token'
);
