// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-proxy-object-internal-methods-and-internal-slots-defineownproperty-p-desc
description: >
  Property descriptor object is created in the Realm of the current execution
  context
info: |
  [[DefineOwnProperty]] (P, Desc)

  ...
  8. Let descObj be FromPropertyDescriptor(Desc).
  9. Let booleanTrapResult be ToBoolean(? Call(trap, handler, « target, P,
     descObj »)).
  ...

  6.2.4.4 FromPropertyDescriptor

  ...
  2. Let obj be ObjectCreate(%ObjectPrototype%).
  ...
  11. Return obj.
features: [cross-realm, Proxy]
---*/

var OProxy = $262.createRealm().global.Proxy;
var desc;
var p = new OProxy({}, {
  defineProperty: function(_, __, _desc) {
    desc = _desc;
    return desc;
  }
});

p.a = 0;

assert.sameValue(Object.getPrototypeOf(desc), Object.prototype);
