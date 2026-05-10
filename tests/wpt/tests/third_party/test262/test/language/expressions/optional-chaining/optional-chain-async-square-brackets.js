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
        OptionalChain ?.[Expression]
features: [optional-chaining]
flags: [async]
includes: [asyncHelpers.js]
---*/

async function checkAssertions() {
  assert.sameValue(await [11]?.[0], 11);
  assert.sameValue([22, 33]?.[await Promise.resolve(1)], 33);
  assert.sameValue([44, await Promise.resolve(55)]?.[1], 55);
  assert.sameValue(undefined?.[
    await Promise.reject(new Error('unreachable'))
  ], undefined);
}
asyncTest(checkAssertions);
