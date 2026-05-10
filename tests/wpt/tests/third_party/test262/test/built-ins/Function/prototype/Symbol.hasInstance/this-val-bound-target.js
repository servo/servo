// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 19.2.3.6
description: Invoked on a bound function
info: |
    1. Let F be the this value.
    2. Return OrdinaryHasInstance(F, V).

    7.3.19 OrdinaryHasInstance (C, O)

    1. If IsCallable(C) is false, return false.
    2. If C has a [[BoundTargetFunction]] internal slot, then
       a. Let BC be the value of C’s [[BoundTargetFunction]] internal slot.
       b. Return InstanceofOperator(O,BC) (see 12.9.4).
features: [Symbol.hasInstance]
---*/

var BC = function() {};
var bc = new BC();
var bound = BC.bind();

assert.sameValue(bound[Symbol.hasInstance](bc), true);
