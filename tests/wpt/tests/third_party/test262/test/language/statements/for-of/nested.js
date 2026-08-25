// Copyright (C) 2013 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 13.6.4.13
description: >
    Nested statements should operate independently.
features: [generators]
---*/

function* values() {
  yield 3;
  yield 7;
}

var outerIterable, expectedOuter, i, innerIterable, expectedInner, j;

outerIterable = values();
expectedOuter = 3;
i = 0;

for (var x of outerIterable) {
  assert.sameValue(x, expectedOuter);
  expectedOuter = 7;
  i++;

  innerIterable = values();
  expectedInner = 3;
  j = 0;
  for (var y of innerIterable) {
    assert.sameValue(y, expectedInner);
    expectedInner = 7;
    j++;
  }

  assert.sameValue(j, 2);
}

assert.sameValue(i, 2);
