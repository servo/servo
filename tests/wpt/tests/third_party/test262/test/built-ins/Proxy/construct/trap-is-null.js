// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-proxy-object-internal-methods-and-internal-slots-construct-argumentslist-newtarget
description: >
    If the construct trap value is null, propagate the construct to the target object.
info: |
    [[Construct]] (argumentsList, newTarget)

    ...
    5. Let trap be ? GetMethod(handler, "construct").
    6. If trap is undefined, then
      a. Assert: target has a [[Construct]] internal method.
      b. Return ? Construct(target, argumentsList, newTarget).
    ...

    GetMethod ( V, P )

    ...
    3. If func is either undefined or null, return undefined.
    ...
features: [Proxy, Reflect, Reflect.construct]
---*/

var calls = 0;
var _NewTarget;

var Target = new Proxy(function() {
  throw new Test262Error('target should not be called');
}, {
  construct: function(_Target, args, NewTarget) {
    calls += 1;
    _NewTarget = NewTarget;
    return {
      sum: args[0] + args[1]
    };
  }
})

var P = new Proxy(Target, {
  construct: null
});

var NewTarget = function() {};
var obj = Reflect.construct(P, [3, 4], NewTarget);

assert.sameValue(calls, 1, "construct is null: [[Construct]] is invoked once");
assert.sameValue(_NewTarget, NewTarget, "construct is null: NewTarget is passed to [[Construct]]");
assert.sameValue(obj.sum, 7, "construct is null: result of [[Construct]] is returned");
