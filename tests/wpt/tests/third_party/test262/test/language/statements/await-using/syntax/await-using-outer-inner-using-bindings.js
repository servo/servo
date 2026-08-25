// Copyright (C) 2023 Ron Buckton. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-runtime-semantics-fordeclarationbindinginstantiation
description: >
    outer await using binding unchanged by for-loop await using binding
flags: [async]
includes: [asyncHelpers.js]
features: [explicit-resource-management]
---*/

asyncTest(async function () {
  const outer_x = { [Symbol.dispose]() {} };
  const outer_y = { [Symbol.dispose]() {} };
  const inner_x = { [Symbol.dispose]() {} };
  const inner_y = { [Symbol.dispose]() {} };

  {
    await using x = outer_x;
    await using y = outer_y;
    var i = 0;

    for (await using x = inner_x; i < 1; i++) {
      await using y = inner_y;

      assert.sameValue(x, inner_x);
      assert.sameValue(y, inner_y);
    }
    assert.sameValue(x, outer_x);
    assert.sameValue(y, outer_y);
  }
});
