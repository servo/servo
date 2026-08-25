// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-proxy-object-internal-methods-and-internal-slots-construct-argumentslist-newtarget
description: >
    trap is called with handler object as its context, and parameters are:
    target, an array list with the called arguments and the new target, and the
    constructor new.target.
info: |
    [[Construct]] ( argumentsList, newTarget)

    9. Let newObj be Call(trap, handler, «target, argArray, newTarget »).
features: [Proxy]
---*/

var _target, _handler, _args, _P;

function Target() {}

var handler = {
  construct: function(t, args, newTarget) {
    _handler = this;
    _target = t;
    _args = args;
    _P = newTarget;

    return new t(args[0], args[1]);
  }
};
var P = new Proxy(Target, handler);

new P(1, 2);

assert.sameValue(_handler, handler, "trap context is the handler object");
assert.sameValue(_target, Target, "first parameter is the target object");
assert.sameValue(_args.length, 2, "arguments list contains all call arguments");
assert.sameValue(_args[0], 1, "arguments list has first call argument");
assert.sameValue(_args[1], 2, "arguments list has second call argument");
assert.sameValue(_P, P, "constructor is sent as the third parameter");
