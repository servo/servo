// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 9.5.1
description: >
    Trap is called with handler as context and target as the first parameter.
info: |
    [[GetPrototypeOf]] ( )

    ...
    8. Let handlerProto be Call(trap, handler, «target»).
    ...

features: [Proxy]
---*/

var _handler, _target;
var target = {};
var handler = {
  getPrototypeOf: function(t) {
    _handler = this;
    _target = t;
    return {};
  }
};

var p = new Proxy(target, handler);

Object.getPrototypeOf(p);

assert.sameValue(_handler, handler);
assert.sameValue(_target, target);
