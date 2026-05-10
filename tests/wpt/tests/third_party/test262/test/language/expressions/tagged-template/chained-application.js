// Copyright (C) 2014 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 12.3.7
description: >
    Tagged templates may be chained and are applied in a left-to-right order.
---*/

var callCount = 0;
var expected = ['x', 'y', 'z'];
var tag = function(templateObject) {
  assert.sameValue(templateObject[0], expected[callCount]);
  callCount++;
  return tag;
}

var result = tag`x``y``z`;

assert.sameValue(callCount, 3);
assert.sameValue(result, tag);
