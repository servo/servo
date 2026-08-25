// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-proxy-object-internal-methods-and-internal-slots-ownpropertykeys
description: >
    [[OwnPropertyKeys]] ( )

    7. Let trapResultArray be ? Call(trap, handler, « target »).
features: [Proxy]
---*/

var _target, _handler;
var target = {
  foo: 1,
  bar: 2
};

var handler = {
  ownKeys: function(t) {
    _handler = this;
    _target = t;
    return Object.getOwnPropertyNames(t);
  }
}
var p = new Proxy(target, handler);

var names = Object.getOwnPropertyNames(p);

assert.sameValue(names[0], "foo");
assert.sameValue(names[1], "bar");
assert.sameValue(names.length, 2);
assert.sameValue(_handler, handler);
assert.sameValue(_target, target);
