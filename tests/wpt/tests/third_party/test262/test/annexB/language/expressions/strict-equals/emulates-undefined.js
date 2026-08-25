// Copyright (C) 2020 Alexey Shvayka. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-equality-operators-runtime-semantics-evaluation
description: >
  Strict Equality Comparison doesn't special-case [[IsHTMLDDA]] objects.
info: |
  EqualityExpression : EqualityExpression === RelationalExpression 

  [...]
  5. Return the result of performing Strict Equality Comparison rval === lval.

  Strict Equality Comparison

  1. If Type(x) is different from Type(y), return false.
features: [IsHTMLDDA]
---*/

var IsHTMLDDA = $262.IsHTMLDDA;

assert.sameValue(IsHTMLDDA === undefined, false, "=== with `undefined`");
assert.sameValue(undefined === IsHTMLDDA, false, "=== with `undefined`");

assert.sameValue(IsHTMLDDA === null, false, "=== with `null`");
assert.sameValue(null === IsHTMLDDA, false, "=== with `null`");

assert(IsHTMLDDA === IsHTMLDDA);
