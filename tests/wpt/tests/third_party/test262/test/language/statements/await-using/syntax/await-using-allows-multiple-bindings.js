// Copyright (C) 2023 Ron Buckton. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-let-const-using-and-await-using-declarations
description: >
    'await using' allows multiple lexical bindings
features: [explicit-resource-management]
---*/

async function f() {
  await using x = null, y = null;
}
