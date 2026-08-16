// Copyright (C) 2023 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
description: https://github.com/tc39/proposal-set-methods/pull/102
features: [set-methods]
---*/

let firstSet = new Set('a', 'b');
let secondSet = {
  size: 3,
  has() {
    firstSet.delete('b');
    firstSet.add('c');
    return true;
  },
  * keys() {}
};
assert.sameValue(firstSet.isSubsetOf(secondSet), true);
