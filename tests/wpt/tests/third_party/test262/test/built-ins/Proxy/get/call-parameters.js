// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 9.5.8
description: >
    [[Get]] (P, Receiver)

    9. Let trapResult be Call(trap, handler, «target, P, Receiver»).
info: |
    6.1.7.2 Object Internal Methods and Internal Slots

    (...) Receiver is used as the this value when evaluating the code
features: [Proxy]
---*/

var _target, _handler, _prop, _receiver;
var target = {
  attr: 1
};
var handler = {
  get: function(t, prop, receiver) {
    _handler = this;
    _target = t;
    _prop = prop;
    _receiver = receiver;
  }
};
var p = new Proxy(target, handler);

p.attr;

assert.sameValue(_handler, handler);
assert.sameValue(_target, target);
assert.sameValue(_prop, "attr");
assert.sameValue(_receiver, p, "receiver is the Proxy object");

_prop = null;
p["attr"];
assert.sameValue(
  _prop, "attr",
  "trap is triggered both by p.attr and p['attr']"
);
