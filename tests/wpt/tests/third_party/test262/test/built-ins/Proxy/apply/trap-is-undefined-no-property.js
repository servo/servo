// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-proxy-object-internal-methods-and-internal-slots-call-thisargument-argumentslist
description: >
    If trap is not set, propagate the call to the target object.
info: |
    [[Call]] (thisArgument, argumentsList)

    ...
    5. Let trap be ? GetMethod(handler, "apply").
    6. If trap is undefined, then
      a. Return ? Call(target, thisArgument, argumentsList).
    ...

    GetMethod ( V, P )

    ...
    3. If func is either undefined or null, return undefined.
    ...
features: [Proxy]
---*/

var calls = 0;
var _context;

var target = new Proxy(function() {}, {
  apply: function(_target, context, args) {
    calls++;
    _context = context;
    return args[0] + args[1];
  }
})

var p = new Proxy(target, {});
var context = {};
var res = p.call(context, 1, 2);

assert.sameValue(calls, 1, "apply is missing: [[Call]] is invoked once");
assert.sameValue(_context, context, "apply is missing: context is passed to [[Call]]");
assert.sameValue(res, 3, "apply is missing: result of [[Call]] is returned");
