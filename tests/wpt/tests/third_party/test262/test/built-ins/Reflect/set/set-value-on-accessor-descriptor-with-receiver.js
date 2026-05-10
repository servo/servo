// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 26.1.13
description: >
  Set value on an accessor descriptor property with receiver as `this`.
info: |
  26.1.13 Reflect.set ( target, propertyKey, V [ , receiver ] )

  ...
  4. If receiver is not present, then
    a. Let receiver be target.
  5. Return target.[[Set]](key, V, receiver).

  9.1.9 [[Set]] ( P, V, Receiver)

  ...
  6. Assert: IsAccessorDescriptor(ownDesc) is true.
  7. Let setter be ownDesc.[[Set]].
  8. If setter is undefined, return false.
  9. Let setterResult be Call(setter, Receiver, «V»).
  10. ReturnIfAbrupt(setterResult).
  11. Return true.
features: [Reflect, Reflect.set]
---*/

var args;
var count = 0
var o1 = {};
var receiver = {};
var _receiver;
Object.defineProperty(o1, 'p', {
  set: function(a) {
    count++;
    args = arguments;
    _receiver = this;
    return false;
  }
});

var result = Reflect.set(o1, 'p', 42, receiver);
assert.sameValue(
  result, true,
  'returns true if target.p has an accessor descriptor'
);
assert.sameValue(args.length, 1, 'target.p set is called with 1 argument');
assert.sameValue(args[0], 42, 'target.p set is called with V');
assert.sameValue(count, 1, 'target.p set is called once');
assert.sameValue(
  _receiver, receiver,
  'target.p set is called with receiver as `this`'
);
