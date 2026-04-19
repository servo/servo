// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 26.1.13
description: >
  Call accessor's set from target's prototype.
info: |
  26.1.13 Reflect.set ( target, propertyKey, V [ , receiver ] )

  ...
  4. If receiver is not present, then
    a. Let receiver be target.
  5. Return target.[[Set]](key, V, receiver).

  9.1.9 [[Set]] ( P, V, Receiver)

  ...
  4. If ownDesc is undefined, then
    a. Let parent be O.[[GetPrototypeOf]]().
    b. ReturnIfAbrupt(parent).
    c. If parent is not null, then
      i. Return parent.[[Set]](P, V, Receiver).
  ...
  11. Return true.
features: [Reflect, Reflect.set]
---*/

var args;
var count = 0;
var _this;
var proto = {};
Object.defineProperty(proto, 'p', {
  set: function() {
    _this = this;
    args = arguments;
    count++;
  }
});

var target = Object.create(proto);
var result = Reflect.set(target, 'p', 42);
assert.sameValue(result, true, 'returns true');
assert.sameValue(args.length, 1, 'prototype `set` called with 1 argument');
assert.sameValue(args[0], 42, 'prototype `set` called with 42');
assert.sameValue(_this, target, 'prototype `set` called with target as `this`');
assert.sameValue(count, 1, 'prototype `set` called once');
