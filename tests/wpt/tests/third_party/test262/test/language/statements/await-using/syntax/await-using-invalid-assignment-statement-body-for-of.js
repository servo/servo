// Copyright (C) 2023 Ron Buckton. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-declarative-environment-records-setmutablebinding-n-v-s
description: >
    await using: invalid assignment in Statement body. Since an `await using` declaration introduces an immutable
    binding, any attempt to change it results in a TypeError.
flags: [async]
includes: [asyncHelpers.js]
features: [explicit-resource-management]
---*/

asyncTest(async function () {
  await assert.throwsAsync(TypeError, async function () {
    for (await using x of [null]) { x = { [Symbol.dispose]() { } }; }
  });
});
