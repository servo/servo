// Copyright 2019 Google, Inc.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: prod-OptionalExpression
description: >
  optional chain bracket notation containing optional expresion
info: |
  OptionalChain:
    ?. [OptionalExpression]
features: [optional-chaining]
---*/
const a = undefined;
const b = {e: 0};
const c = {};
c[undefined] = 11;
const d = [22];

assert.sameValue(undefined, a?.[a?.b]);
assert.sameValue(11, c?.[a?.b]);
assert.sameValue(22, d?.[b?.e]);
