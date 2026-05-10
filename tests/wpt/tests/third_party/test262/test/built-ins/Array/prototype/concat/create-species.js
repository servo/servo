// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-array.prototype.concat
description: Species constructor is used to create a new instance
info: |
    1. Let O be ? ToObject(this value).
    2. Let A be ? ArraySpeciesCreate(O, 0).

    9.4.2.3 ArraySpeciesCreate

    [...]
    5. Let C be ? Get(originalArray, "constructor").
    [...]
    7. If Type(C) is Object, then
       a. Let C be ? Get(C, @@species).
       b. If C is null, let C be undefined.
    [...]
    10. Return ? Construct(C, « length »).
features: [Symbol.species]
---*/

var thisValue, args, result;
var callCount = 0;
var instance = [];
var Ctor = function() {
  callCount += 1;
  thisValue = this;
  args = arguments;
  return instance;
};
var a = [];
a.constructor = {};
a.constructor[Symbol.species] = Ctor;

result = a.concat();

assert.sameValue(callCount, 1, 'The value of callCount is expected to be 1');
assert.sameValue(
  Object.getPrototypeOf(thisValue),
  Ctor.prototype,
  'Object.getPrototypeOf(this) returns Ctor.prototype'
);
assert.sameValue(args.length, 1, 'The value of args.length is expected to be 1');
assert.sameValue(args[0], 0, 'The value of args[0] is expected to be 0');
assert.sameValue(result, instance, 'The value of result is expected to equal the value of instance');
