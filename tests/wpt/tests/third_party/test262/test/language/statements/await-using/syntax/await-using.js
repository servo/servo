// Copyright (C) 2023 Ron Buckton. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-let-const-using-and-await-using-declarations
description: >
    module and block scope await using
flags: [module]
features: [explicit-resource-management]
---*/

await using z = null;

// Block local
{
  await using z = undefined;
}

assert.sameValue(z, null);

if (true) {
  const obj = { [Symbol.dispose]() { } };
  await using z = obj;
  assert.sameValue(z, obj);
}

