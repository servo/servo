// Copyright 2019 Google, Inc.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: prod-OptionalExpression
description: >
  optional chain on member expression in async context
info: |
  Left-Hand-Side Expressions
    OptionalExpression
      MemberExpression [PrimaryExpression identifier] OptionalChain
features: [optional-chaining]
flags: [async]
includes: [asyncHelpers.js]
---*/

const a = undefined;
const c = {d: Promise.resolve(11)};
async function checkAssertions() {
  assert.sameValue(await a?.b, undefined);
  assert.sameValue(await c?.d, 11);
  
  Promise.prototype.x = 42;
  var res = await Promise.resolve(undefined)?.x;
  assert.sameValue(res, 42, 'await unwraps the evaluation of the whole optional chaining expression #1');

  Promise.prototype.y = 43;
  var res = await Promise.reject(undefined)?.y;
  assert.sameValue(res, 43, 'await unwraps the evaluation of the whole optional chaining expression #2');
  
  c.e = Promise.resolve(39);
  assert.sameValue(await c?.e, 39, 'await unwraps the promise given after the evaluation of the OCE');
}
asyncTest(checkAssertions);
