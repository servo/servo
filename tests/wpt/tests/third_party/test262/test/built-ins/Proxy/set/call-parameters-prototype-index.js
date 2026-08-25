// Copyright (C) 2019 Alexey Shvayka. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-proxy-object-internal-methods-and-internal-slots-set-p-v-receiver
description: >
  Ordinary [[Set]] forwards call to Proxy "set" trap with correct arguments.
  (integer index property name)
info: |
  OrdinarySet ( O, P, V, Receiver )

  ...
  3. Return OrdinarySetWithOwnDescriptor(O, P, V, Receiver, ownDesc).

  OrdinarySetWithOwnDescriptor ( O, P, V, Receiver, ownDesc )

  ...
  2. If ownDesc is undefined, then
    a. Let parent be ? O.[[GetPrototypeOf]]().
    b. If parent is not null, then
      i. Return ? parent.[[Set]](P, V, Receiver).

  [[Set]] ( P, V, Receiver )

  ...
  8. Let booleanTrapResult be ! ToBoolean(? Call(trap, handler, « target, P, V, Receiver »)).
  ...
  12. Return true.
includes: [proxyTrapsHelper.js]
features: [Proxy]
---*/

var _handler, _target, _prop, _value, _receiver;
var target = {};
var handler = allowProxyTraps({
  set: function(target, prop, value, receiver) {
    _handler = this;
    _target = target;
    _prop = prop;
    _value = value;
    _receiver = receiver;
    return true;
  },
});

var proxy = new Proxy(target, handler);
var array = new Array(1);
Object.setPrototypeOf(array, proxy);

array[0] = 1;

assert.sameValue(_handler, handler, 'handler object is the trap context');
assert.sameValue(_target, target, 'first argument is the target object');
assert.sameValue(_prop, '0', 'second argument is the property name');
assert.sameValue(_value, 1, 'third argument is the new value');
assert.sameValue(_receiver, array, 'fourth argument is the receiver object');
