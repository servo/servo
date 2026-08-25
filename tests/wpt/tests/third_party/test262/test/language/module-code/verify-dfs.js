// Copyright (C) 2020 Ecma International. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: Dynamic import can't preempt DFS evaluation order
esid: sec-moduleevaluation
info: |
  Evaluate ( ) Concrete Method

  1. Assert: This call to Evaluate is not happening at the same time as another call to Evaluate within the surrounding agent.
  [...]
flags: [module, async]
features: [dynamic-import]
---*/

import './verify-dfs-a_FIXTURE.js';
import './verify-dfs-b_FIXTURE.js';

// rely on function hoisting to create shared array
export function evaluated(name) {
  if (!evaluated.order) {
    evaluated.order = [];
  }
  evaluated.order.push(name);
}

export function check(promise) {
  promise.then(() => {
    assert.sameValue(evaluated.order.length, 2);
    assert.sameValue(evaluated.order[0], 'A');
    assert.sameValue(evaluated.order[1], 'B');
  })
  .then($DONE, $DONE);
}
