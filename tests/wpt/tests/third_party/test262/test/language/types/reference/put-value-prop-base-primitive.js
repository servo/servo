// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-putvalue
es6id: 6.2.3.2
description: >
  When the base of a property reference is primitive, it is coerced to an
  object during value assignment
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
features: [Symbol, Proxy]
---*/

var numberCount = 0;
var stringCount = 0;
var booleanCount = 0;
var symbolCount = 0;
var spy;

spy = new Proxy({}, { set: function() { numberCount += 1; return true; } });
Object.setPrototypeOf(Number.prototype, spy);
0..test262 = null;
assert.sameValue(numberCount, 1, 'number');

spy = new Proxy({}, { set: function() { stringCount += 1; return true; } });
Object.setPrototypeOf(String.prototype, spy);
''.test262 = null;
assert.sameValue(stringCount, 1, 'string');

spy = new Proxy({}, { set: function() { booleanCount += 1; return true; } });
Object.setPrototypeOf(Boolean.prototype, spy);
true.test262 = null;
assert.sameValue(booleanCount, 1, 'string');

spy = new Proxy({}, { set: function() { symbolCount += 1; return true; } });
Object.setPrototypeOf(Symbol.prototype, spy);
Symbol().test262 = null;
assert.sameValue(symbolCount, 1, 'string');
