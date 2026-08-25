// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-runtime-semantics-catchclauseevaluation
description: Creation of new lexical environment for `catch` parameter
---*/

var probeBefore = function() { return x; };
var probeTry, probeParam;
var x = 'outside';

try {
  probeTry = function() { return x; };

  throw ['inside'];
} catch ([x, _ = probeParam = function() { return x; }]) {}

assert.sameValue(probeBefore(), 'outside');
assert.sameValue(probeTry(), 'outside');
assert.sameValue(probeParam(), 'inside');
