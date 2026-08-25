// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 26.1.9
description: >
  Return boolean value.
info: |
  26.1.9 Reflect.has ( target, propertyKey )

  ...
  4. Return target.[[HasProperty]](key).


  9.1.7.1 OrdinaryHasProperty (O, P)

  ...
  2. Let hasOwn be OrdinaryGetOwnProperty(O, P).
  3. If hasOwn is not undefined, return true.
  4. Let parent be O.[[GetPrototypeOf]]().
  5. ReturnIfAbrupt(parent).
  6. If parent is not null, then
    a. Return parent.[[HasProperty]](P).
  7. Return false.
features: [Reflect]
---*/

var o1 = {
  p: 42
};

assert.sameValue(Reflect.has(o1, 'p'), true, 'true from own property');
assert.sameValue(
  Reflect.has(o1, 'z'), false,
  'false when property is not present'
);

var o2 = Object.create({
  p: 42
});
assert.sameValue(Reflect.has(o2, 'p'), true, 'true from a prototype property');
