// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-proxy-object-internal-methods-and-internal-slots-call-thisargument-argumentslist
description: >
    trap is called with handler object as its context, and parameters are:
    target, the call context and and an array list with the called arguments
info: |
    [[Call]] (thisArgument, argumentsList)

    9. Return Call(trap, handler, «target, thisArgument, argArray»).
features: [Proxy]
---*/

var _target, _args, _handler, _context;
var target = function() {
  throw new Test262Error('target should not be called');
};
var handler = {
  apply: function(t, c, args) {
    _handler = this;
    _target = t;
    _context = c;
    _args = args;
  }
};
var p = new Proxy(target, handler);

var context = {};

p.call(context, 1, 2);

assert.sameValue(_handler, handler, "trap context is the handler object");
assert.sameValue(_target, target, "first parameter is the target object");
assert.sameValue(_context, context, "second parameter is the call context");
assert.sameValue(_args.length, 2, "arguments list contains all call arguments");
assert.sameValue(_args[0], 1, "arguments list has first call argument");
assert.sameValue(_args[1], 2, "arguments list has second call argument");
