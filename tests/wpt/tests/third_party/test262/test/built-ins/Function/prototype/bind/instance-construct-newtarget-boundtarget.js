// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-bound-function-exotic-objects-construct-argumentslist-newtarget
description: >
  The NewTarget value is changed to the target function when the bound function
  object is constructed using Reflect.construct and the "bound target" is
  specified as the NewTarget value
info: |
  [...]
  5. If SameValue(F, newTarget) is true, let newTarget be target.
  6. Return ? Construct(target, args, newTarget). 
features: [Reflect, new.target]
---*/

var newTarget;
function A() {
  newTarget = new.target;
}
var B = A.bind();
var C = B.bind();

var c = Reflect.construct(C, [], A);

assert.sameValue(newTarget, A);
assert.sameValue(Object.getPrototypeOf(c), A.prototype);
