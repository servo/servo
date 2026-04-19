// Copyright 2019 Google, Inc.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: prod-OptionalExpression
description: >
  optional chain expansions in an async context
info: |
  Left-Hand-Side Expressions
    OptionalExpression
      MemberExpression [PrimaryExpression Identifier] OptionalChain
        OptionalChain OptionalChain ?.[Expression]
features: [optional-chaining]
flags: [async]
includes: [asyncHelpers.js]
---*/

async function checkAssertions() {
  assert.sameValue(await {a: [11]}?.a[0], 11);
  const b = {c: [22, 33]};
  assert.sameValue(b?.c[await Promise.resolve(1)], 33);
  function e(val) {
    return val;
  }
  assert.sameValue({d: e}?.d(await Promise.resolve([44, 55]))[1], 55);
  assert.sameValue(undefined?.arr[
    await Promise.reject(new Error('unreachable'))
  ], undefined);
}
asyncTest(checkAssertions);
