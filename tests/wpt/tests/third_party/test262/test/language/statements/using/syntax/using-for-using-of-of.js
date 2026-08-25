// Copyright (C) 2023 Ron Buckton. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-for-in-and-for-of-statements
description: >
    using: 'for (using of' is always interpreted as identifier 
features: [explicit-resource-management]
---*/

var using, of = [[9], [8], [7]], result = [];
for (using of of [0, 1, 2]) {
  // ^^^^^    ^^^^^^^^^^^^
  // |        |
  // |        interpreted as element access `of[2]`
  // |
  // interpreted as identifier named `using`

  result.push(using);
}

assert.sameValue(result.length, 1);
assert.sameValue(result[0], 7);
