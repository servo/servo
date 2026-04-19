// Copyright 2019 Google, Inc.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: prod-OptionalExpression
description: >
  optional chain on member expression in async context
info: |
  Left-Hand-Side Expressions
    OptionalExpression:
      MemberExpression [PrimaryExpression this] OptionalChain
features: [optional-chaining]
flags: [async]
---*/

async function thisFn() {
  return await this?.a
}
thisFn.call({a: Promise.resolve(33)}).then(function(arg) {
  assert.sameValue(33, arg);
}).then($DONE, $DONE);
