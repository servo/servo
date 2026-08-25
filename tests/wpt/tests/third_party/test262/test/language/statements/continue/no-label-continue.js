// Copyright (C) 2014 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 13.12
description: >
    no label continue
---*/
var count = 0;
for (let x = 0; x < 10;) {
  x++;
  count++;
  continue;
}
assert.sameValue(count, 10, "The value of `count` is `10`");

