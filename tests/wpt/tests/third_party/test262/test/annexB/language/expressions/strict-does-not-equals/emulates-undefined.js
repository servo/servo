// Copyright (C) 2020 Alexey Shvayka. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-equality-operators-runtime-semantics-evaluation
description: >
  Strict Equality Comparison doesn't special-case [[IsHTMLDDA]] objects.
info: |
  EqualityExpression : EqualityExpression !== RelationalExpression 

  [...]
  5. Let r be the result of performing Strict Equality Comparison rval === lval.
  6. Assert: r is a normal completion.
  7. If r.[[Value]] is true, return false. Otherwise, return true.

  Strict Equality Comparison

  1. If Type(x) is different from Type(y), return false.
features: [IsHTMLDDA]
---*/

var IsHTMLDDA = $262.IsHTMLDDA;

assert(IsHTMLDDA !== undefined, "!== with `undefined`");
assert(undefined !== IsHTMLDDA, "!== with `undefined`");

assert(IsHTMLDDA !== null, "!== with `null`");
assert(null !== IsHTMLDDA, "!== with `null`");

assert.sameValue(IsHTMLDDA !== IsHTMLDDA, false);
