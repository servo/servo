// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-super-keyword
es6id: 12.3.5
description: Invocation of "parent" constructor
info: |
  [...]
  6. Let result be ? Construct(func, argList, newTarget).
  [...]
features: [class, new.target, Reflect, Reflect.construct]
---*/

var expectedNewTarget = function() {};
var thisValue, instance, args, actualNewTarget;
function Parent() {
  thisValue = this;
  args = arguments;
  actualNewTarget = new.target;
}

class Child extends Parent {
  constructor() {
    super(1, 2, 3);
  }
}

instance = Reflect.construct(Child, [4, 5, 6], expectedNewTarget);

assert.sameValue(thisValue, instance);
assert.sameValue(args.length, 3, 'length of provided arguments object');
assert.sameValue(args[0], 1, 'first argument');
assert.sameValue(args[1], 2, 'second argument');
assert.sameValue(args[2], 3, 'third argument');
assert.sameValue(actualNewTarget, expectedNewTarget, 'new.target value');
