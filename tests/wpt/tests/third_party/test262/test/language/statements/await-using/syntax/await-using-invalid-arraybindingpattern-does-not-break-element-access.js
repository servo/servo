// Copyright (C) 2023 Ron Buckton. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-let-const-using-and-await-using-declarations
description: >
    'await using' does not break existing element access
flags: [async]
includes: [asyncHelpers.js]
features: [explicit-resource-management]
---*/

var using = [], x = 0;

asyncTest(async function () {
  await using[x];
});
