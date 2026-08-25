// Copyright (C) 2019 Alexey Shvayka. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-proxy-object-internal-methods-and-internal-slots-hasproperty-p
description: >
  Ordinary [[HasProperty]] forwards call to Proxy "has" trap with correct arguments.
  (integer index property name)
info: |
  OrdinaryHasProperty ( O, P )

  ...
  4. Let parent be ? O.[[GetPrototypeOf]]().
  5. If parent is not null, then
    a. Return ? parent.[[HasProperty]](P).

  [[HasProperty]] ( P )

  ...
  8. Let booleanTrapResult be ! ToBoolean(? Call(trap, handler, « target, P »)).
  ...
  10. Return booleanTrapResult.
includes: [proxyTrapsHelper.js]
features: [Proxy]
---*/

var _handler, _target, _prop;
var proto = [14];
var target = Object.create(proto);
var handler = allowProxyTraps({
  has: function(target, prop) {
    _handler = this;
    _target = target;
    _prop = prop;

    return false;
  },
});
var proxy = new Proxy(target, handler);
var array = [];
Object.setPrototypeOf(array, proxy);

assert.sameValue(1 in array, false);
assert.sameValue(_handler, handler, 'handler is context');
assert.sameValue(_target, target, 'target is the first parameter');
assert.sameValue(_prop, '1', 'given prop is the second paramter');
