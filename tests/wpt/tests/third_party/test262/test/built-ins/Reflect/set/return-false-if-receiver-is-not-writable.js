// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 26.1.13
description: >
  Return false if receiver is not writable.
info: |
  26.1.13 Reflect.set ( target, propertyKey, V [ , receiver ] )

  ...
  4. If receiver is not present, then
    a. Let receiver be target.
  5. Return target.[[Set]](key, V, receiver).

  9.1.9 [[Set]] ( P, V, Receiver)

  ...
  5. If IsDataDescriptor(ownDesc) is true, then
    a. If ownDesc.[[Writable]] is false, return false.
  ...
features: [Reflect, Reflect.set]
---*/

var o1 = {};

Object.defineProperty(o1, 'p', {
  writable: false,
  value: 42
});
var result = Reflect.set(o1, 'p', 43);
assert.sameValue(result, false, 'returns false');
assert.sameValue(o1.p, 42, 'does not set a new value');
