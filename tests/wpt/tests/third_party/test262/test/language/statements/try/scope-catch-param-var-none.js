// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-runtime-semantics-catchclauseevaluation
description: Retainment of existing variable environment for `catch` parameter
flags: [noStrict]
---*/

var x = 1;
var probeBefore = function() { return x; };
var probeTry, probeParam, probeBlock;

try {
  var x = 2;
  probeTry = function() { return x; };
  throw [];
} catch ([_ = (eval('var x = 3;'), probeParam = function() { return x; })]) {
  var x = 4;
  probeBlock = function() { return x; };
}

assert.sameValue(probeBefore(), 4, 'reference preceding statement');
assert.sameValue(probeTry(), 4, 'reference from `try` block');
assert.sameValue(probeParam(), 4, 'reference within CatchParameter');
assert.sameValue(probeBlock(), 4, 'reference from `catch` block');
assert.sameValue(x, 4, 'reference following statement');
