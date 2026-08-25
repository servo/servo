// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 19.1.2.1
description: Does not assign non-enumerable source properties
info: |
    [...]
    5. For each element nextSource of sources, in ascending index order,
       c. Repeat for each element nextKey of keys in List order,
          i. Let desc be from.[[GetOwnProperty]](nextKey).
          ii. ReturnIfAbrupt(desc).
          iii. if desc is not undefined and desc.[[Enumerable]] is true, then
---*/

var target = {};
var source = Object.defineProperty({}, 'attr', {
  value: 1,
  enumerable: false
});
var result;

result = Object.assign(target, source);

assert(
  !Object.prototype.hasOwnProperty.call(target, 'attr'),
  "Non-enumerable property 'attr' does not get copied to target"
);
assert.sameValue(result, target);
