// Copyright 2019 Google, Inc.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: prod-OptionalExpression
description: >
  various optional chain expansions
info: |
  OptionalChain[Yield, Await]:
    ?.[Expression]
    ?.IdentifierName
    ?.Arguments
    ?.TemplateLiteral
    OptionalChain [Expression]
    OptionalChain .IdentifierName
    OptionalChain Arguments[?Yield, ?Await]
    OptionalChain TemplateLiteral
features: [optional-chaining]
---*/

const arr = [10, 11];
const obj = {
  a: 'hello',
  b: {val: 13},
  c(arg1) {
    return arg1 * 2;
  },
  arr: [11, 12]
};
const i = 0;

// OptionalChain: ?.[Expression]
assert.sameValue(11, arr?.[i + 1]);

// OptionalChain: ?.IdentifierName
assert.sameValue('hello', obj?.a);

// OptionalChain: ?.Arguments
const fn = (arg1, arg2) => {
  return arg1 + arg2;
}
assert.sameValue(30, fn?.(10, 20));

// OptionalChain: OptionalChain [Expression]
assert.sameValue(12, obj?.arr[i + 1]);

// OptionalChain: OptionalChain .IdentifierName
assert.sameValue(13, obj?.b.val);

// OptionalChain: OptionalChain Arguments
assert.sameValue(20, obj?.c(10));
