// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-getvalue
es6id: 6.2.3.1
description: >
  When the base of a property reference is primitive, it is coerced to an
  object during value retrieval (honoring the realm of the current execution
  context)
info: |
  [...]
  5. If IsPropertyReference(V) is true, then
     a. If HasPrimitiveBase(V) is true, then
        i. Assert: In this case, base will never be null or undefined.
        ii. Let base be ToObject(base).
     b. Return ? base.[[Get]](GetReferencedName(V), GetThisValue(V)).
features: [cross-realm, Symbol]
---*/

var other = $262.createRealm().global;

other.Number.prototype.test262 = 'number prototype';
other.value = 1;
assert.sameValue(other.eval('value.test262'), 'number prototype');

other.String.prototype.test262 = 'string prototype';
other.value = '';
assert.sameValue(other.eval('value.test262'), 'string prototype');

other.Boolean.prototype.test262 = 'Boolean prototype';
other.value = true;
assert.sameValue(other.eval('value.test262'), 'Boolean prototype');

other.Symbol.prototype.test262 = 'Symbol prototype';
other.value = Symbol();
assert.sameValue(other.eval('value.test262'), 'Symbol prototype');
