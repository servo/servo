// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
includes: [compareArray.js]
description: |
  pending
esid: pending
---*/

var array = ["not-a-number", "also-not-a-number"];
var copy = [...array];

// The sort comparator must be exactly equal to the bytecode pattern:
//
// JSOp::GetArg 0/1
// JSOp::GetArg 1/0
// JSOp::Sub
// JSOp::Return
array.sort(function(a, b) { return a - b; });

assert.compareArray(array, copy);

