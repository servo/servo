// Copyright (C) 2021 Kevin Gibbons. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: for statement beginning with `async of`
info: |
  `for (async of =>` is the begining of a regular for loop, rather than a for-of
esid: sec-for-statement
---*/

var i = 0;
var counter = 0;
for (async of => {}; i < 10; ++i) {
  ++counter;
}
assert.sameValue(counter, 10);
