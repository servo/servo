// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 9.5.3
description: >
    The trap is called with handler on its context and the target object as the
    first parabeter
info: |
    [[IsExtensible]] ( )

    ...
    8. Let booleanTrapResult be ToBoolean(Call(trap, handler, «target»)).
    ...

features: [Proxy]
---*/

var _target, _handler;
var target = {};
var handler = {
  isExtensible: function(t) {
    _handler = this;
    _target = t;
    return Object.isExtensible(t);
  }
}
var p = new Proxy(target, handler);

Object.isExtensible(p);

assert.sameValue(_handler, handler);
assert.sameValue(_target, target);
