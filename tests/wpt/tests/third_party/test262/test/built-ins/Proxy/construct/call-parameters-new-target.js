// Copyright (C) 2017 Aleksey Shvayka. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-proxy-object-internal-methods-and-internal-slots-construct-argumentslist-newtarget
description: >
    trap is called with handler object as its context, and parameters are:
    target, an array list with the called arguments and the NewTarget
info: |
    [[Construct]] (argumentsList, newTarget)

    9. Let newObj be Call(trap, handler, «target, argArray, newTarget»).
features: [Proxy, Reflect, Reflect.construct]
---*/

function Target() {}

function NewTarget() {}

var handler = {
  construct: function(target, args, newTarget) {
    assert.sameValue(this, handler, "trap context is the handler object");
    assert.sameValue(target, Target, "first parameter is the target object");
    assert.sameValue(args.length, 2, "arguments list contains all construct arguments");

    var a = args[0];
    var b = args[1];
    assert.sameValue(a, 1, "arguments list has first construct argument");
    assert.sameValue(b, 2, "arguments list has second construct argument");
    assert.sameValue(newTarget, NewTarget, "newTarget is passed as the third parameter");

    return {
      sum: a + b
    };
  },
};

var P = new Proxy(Target, handler);
var res = Reflect.construct(P, [1, 2], NewTarget);
assert.sameValue(res.sum, 3);
