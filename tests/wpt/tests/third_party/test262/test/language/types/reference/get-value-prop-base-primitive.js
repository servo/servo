// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-getvalue
es6id: 6.2.3.1
description: >
  When the base of a property reference is primitive, it is coerced to an
  object during value retrieval
info: |
  [...]
  5. If IsPropertyReference(V) is true, then
     a. If HasPrimitiveBase(V) is true, then
        i. Assert: In this case, base will never be null or undefined.
        ii. Let base be ToObject(base).
     b. Return ? base.[[Get]](GetReferencedName(V), GetThisValue(V)).
features: [Symbol]
---*/

Number.prototype.test262 = 'number prototype';
assert.sameValue(1..test262, 'number prototype');

String.prototype.test262 = 'string prototype';
assert.sameValue(''.test262, 'string prototype');

Boolean.prototype.test262 = 'Boolean prototype';
assert.sameValue(true.test262, 'Boolean prototype');

Symbol.prototype.test262 = 'Symbol prototype';
assert.sameValue(Symbol().test262, 'Symbol prototype');
