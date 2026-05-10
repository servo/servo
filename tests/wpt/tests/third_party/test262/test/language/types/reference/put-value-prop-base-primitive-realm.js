// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-putvalue
es6id: 6.2.3.2
description: >
  When the base of a property reference is primitive, it is coerced to an
  object during value assignment (honoring the realm of the current execution
  context)
info: |
  [...]
  6. Else if IsPropertyReference(V) is true, then
     a. If HasPrimitiveBase(V) is true, then
        i. Assert: In this case, base will never be null or undefined.
        ii. Set base to ToObject(base).
     b. Let succeeded be ? base.[[Set]](GetReferencedName(V), W,
        GetThisValue(V)).
     c. If succeeded is false and IsStrictReference(V) is true, throw a
        TypeError exception.
     d. Return.
features: [cross-realm, Symbol, Proxy]
---*/

var other = $262.createRealm().global;
var numberCount = 0;
var stringCount = 0;
var booleanCount = 0;
var symbolCount = 0;
var spy;

spy = new Proxy({}, { set: function() { numberCount += 1; return true; } });
Object.setPrototypeOf(other.Number.prototype, spy);
other.eval('0..test262 = null;');
assert.sameValue(numberCount, 1, 'number');

spy = new Proxy({}, { set: function() { stringCount += 1; return true; } });
Object.setPrototypeOf(other.String.prototype, spy);
other.eval('"".test262 = null;');
assert.sameValue(stringCount, 1, 'string');

spy = new Proxy({}, { set: function() { booleanCount += 1; return true; } });
Object.setPrototypeOf(other.Boolean.prototype, spy);
other.eval('true.test262 = null;');
assert.sameValue(booleanCount, 1, 'string');

spy = new Proxy({}, { set: function() { symbolCount += 1; return true; } });
Object.setPrototypeOf(other.Symbol.prototype, spy);
other.eval('Symbol().test262 = null;');
assert.sameValue(symbolCount, 1, 'string');
