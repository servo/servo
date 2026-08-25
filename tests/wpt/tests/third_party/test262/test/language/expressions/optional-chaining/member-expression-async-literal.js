// Copyright 2019 Google, Inc.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: prod-OptionalExpression
description: >
  optional chain on member expression in async context
info: |
  Left-Hand-Side Expressions
    OptionalExpression:
      MemberExpression [PrimaryExpression literal] OptionalChain
features: [optional-chaining]
flags: [async]
includes: [asyncHelpers.js]
---*/

async function checkAssertions() {
  assert.sameValue(await "hello"?.[0], 'h');
  assert.sameValue(await null?.a, undefined);
}
asyncTest(checkAssertions);
