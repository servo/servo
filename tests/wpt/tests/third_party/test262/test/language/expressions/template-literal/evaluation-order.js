// Copyright (C) 2014 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 12.2.8
description: Expressions should be evaluated in left-to-right order.
---*/

var tag = function(templateObject, a, b, c) {
  callCount++;
  assert.sameValue(a, 0);
  assert.sameValue(b, 1);
  assert.sameValue(c, 2);
};
var i = 0;
var callCount;

assert.sameValue(`a${ i++ }b${ i++ }c${ i++ }d`, 'a0b1c2d');

i = 0;
callCount = 0;

tag`a${ i++ }b${ i++ }c${ i++ }d`;

assert.sameValue(callCount, 1);
