// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 19.1.2.1
description: Invoked with a source which does not have a descriptor for an own property
info: |
    [...]
    5. For each element nextSource of sources, in ascending index order,
       [...]
       c. Repeat for each element nextKey of keys in List order,
          i. Let desc be from.[[GetOwnProperty]](nextKey).
          ii. ReturnIfAbrupt(desc).
          iii. if desc is not undefined and desc.[[Enumerable]] is true, then
features: [Proxy]
---*/

var callCount = 0;
var target = {};
var result;
var source = new Proxy({}, {
  ownKeys: function() {
    callCount += 1;
    return ['missing'];
  }
});

result = Object.assign(target, source);

assert.sameValue(callCount, 1, 'Proxy trap was invoked exactly once');
assert(
  !Object.prototype.hasOwnProperty.call(target, 'missing'),
  'An own property was not created for a property without a property descriptor'
);
assert.sameValue(result, target);
