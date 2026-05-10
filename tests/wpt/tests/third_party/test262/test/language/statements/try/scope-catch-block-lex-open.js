// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-runtime-semantics-catchclauseevaluation
description: Creation of new lexical environment for `catch` block
info: |
    [...]
    8. Let B be the result of evaluating Block.
    [...]
features: [let]
---*/

var probeParam, probeBlock;
let x = 'outside';

try {
  throw [];
} catch ([_ = probeParam = function() { return x; }]) {
  probeBlock = function() { return x; };
  let x = 'inside';
}

assert.sameValue(probeParam(), 'outside');
assert.sameValue(probeBlock(), 'inside');
