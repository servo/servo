// Copyright 2019 Google, LLC.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: prod-OptionalExpression
description: >
  optional chain RHS of for await statement
info: |
  IterationStatement
    for await (LeftHandSideExpression of AssignmentExpression) Statement
features: [optional-chaining]
flags: [async]
includes: [asyncHelpers.js]
---*/
const obj = {
  iterable: {
    [Symbol.asyncIterator]() {
      return {
        i: 0,
        next() {
          if (this.i < 3) {
            return Promise.resolve({ value: this.i++, done: false });
          }
          return Promise.resolve({ done: true });
        }
      };
    }
  }
};
async function checkAssertions() {
  let count = 0;
  for await (const num of obj?.iterable) {
    count += num;
  }
  assert.sameValue(3, count);
}
asyncTest(checkAssertions);
