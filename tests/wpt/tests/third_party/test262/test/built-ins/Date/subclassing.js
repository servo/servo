// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-date-value
description: Error retrieving `Symbol.toPrimitive` method from object value
info: |
  3. If NewTarget is not undefined, then
     [...]
     c. Let O be ? OrdinaryCreateFromConstructor(NewTarget, "%DatePrototype%",
        « [[DateValue]] »).
     [...]

  OrdinaryCreateFromConstructor ( constructor, intrinsicDefaultProto [ ,
  internalSlotsList ] )

  [...]
  2. Let proto be ? GetPrototypeFromConstructor(constructor,
     intrinsicDefaultProto).
  3. Return ObjectCreate(proto, internalSlotsList).
features: [Reflect]
---*/

var callCount = 0;
var Ctor = function() {
  callCount += 1;
};
var instance;

instance = Reflect.construct(Date, [64], Ctor);

assert.sameValue(
  Object.getPrototypeOf(instance),
  Ctor.prototype,
  'constructor defines an object `prototype` property'
);
assert.sameValue(callCount, 0, 'constructor not invoked');
assert.sameValue(
  Date.prototype.getTime.call(instance),
  64,
  'proper subclass has a [[DateValue]] slot'
);

Ctor.prototype = null;
instance = Reflect.construct(Date, [64], Ctor);

assert.sameValue(
  Object.getPrototypeOf(instance),
  Date.prototype,
  'constructor does not defines an object `prototype` property'
);
assert.sameValue(callCount, 0, 'constructor not invoked');
assert.sameValue(
  instance.getTime(), 64, 'direct instance has a [[DateValue]] slot'
);
