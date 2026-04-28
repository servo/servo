// Copyright 2020 Salesforce.com, Inc. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: prod-OptionalExpression
description: >
  Productions for ?. IdentifierName
info: |
  OptionalChain[Yield, Await]:
    ?. IdentifierName
features: [optional-chaining]
---*/

const arr = [10, 11];
const obj = {
  a: 'hello'
};

assert.sameValue(obj?.a, 'hello');
assert.sameValue(obj?.\u0061, 'hello');
assert.sameValue(obj?.\u{0061}, 'hello');

assert.sameValue(obj?.\u0062, undefined);
assert.sameValue(obj?.\u{0062}, undefined);

assert.sameValue(arr ?. length, 2);
assert.sameValue(arr ?. l\u0065ngth, 2);
assert.sameValue(arr ?. l\u{0065}ngth, 2);

assert.sameValue(obj?.$, undefined);

obj.$ = 42;
assert.sameValue(obj?.$, 42);

assert.sameValue(obj?._, undefined);

obj._ = 39;
assert.sameValue(obj?._, 39);
