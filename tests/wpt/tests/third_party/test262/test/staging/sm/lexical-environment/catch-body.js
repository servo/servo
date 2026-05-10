// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: |
  pending
esid: pending
---*/
function f() {
  var probeParam, probeBlock;
  let x = 'outside';

  try {
    throw [];
  } catch ([_ = probeParam = function() { return x; }]) {
    probeBlock = function() { return x; };
    let x = 'inside';
  }

  assert.sameValue(probeBlock(), 'inside');
  assert.sameValue(probeParam(), 'outside');
}

f();

