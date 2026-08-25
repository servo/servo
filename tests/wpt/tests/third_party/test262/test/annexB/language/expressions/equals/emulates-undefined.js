// Copyright (C) 2020 Alexey Shvayka. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-equality-operators-runtime-semantics-evaluation
description: >
  Abstract Equality special-cases [[IsHTMLDDA]] objects with `undefined` and `null`.
info: |
  EqualityExpression : EqualityExpression == RelationalExpression

  [...]
  5. Return the result of performing Abstract Equality Comparison rval == lval.

  The [[IsHTMLDDA]] Internal Slot / Changes to Abstract Equality Comparison

  The following steps are inserted after step 3 of the Abstract Equality Comparison algorithm:

  1. If Type(x) is Object and x has an [[IsHTMLDDA]] internal slot and y is either null or undefined, return true.
  2. If x is either null or undefined and Type(y) is Object and y has an [[IsHTMLDDA]] internal slot, return true.
features: [IsHTMLDDA]
---*/

var IsHTMLDDA = $262.IsHTMLDDA;

assert(IsHTMLDDA == undefined, "== with `undefined`");
assert(undefined == IsHTMLDDA, "== with `undefined`");

assert(IsHTMLDDA == null, "== with `null`");
assert(null == IsHTMLDDA, "== with `null`");

assert(IsHTMLDDA == IsHTMLDDA);
