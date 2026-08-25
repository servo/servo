// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: |
  pending
esid: pending
---*/
function boundTarget(expected) {
    assert.sameValue(new.target, expected);
}

let bound = boundTarget.bind(undefined);

const TEST_ITERATIONS = 550;

for (let i = 0; i < TEST_ITERATIONS; i++)
    bound(undefined);

for (let i = 0; i < TEST_ITERATIONS; i++)
    new bound(boundTarget);

