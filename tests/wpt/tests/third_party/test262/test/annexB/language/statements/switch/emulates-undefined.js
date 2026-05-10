// Copyright (C) 2020 Alexey Shvayka. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-runtime-semantics-caseclauseisselected
description: >
  `switch` statement uses Strict Equality Comparison,
  which doesn't special-case [[IsHTMLDDA]] objects.
info: |
  Runtime Semantics: CaseClauseIsSelected ( C, input )

  [...]
  4. Return the result of performing Strict Equality Comparison input === clauseSelector.

  Strict Equality Comparison

  1. If Type(x) is different from Type(y), return false.
features: [IsHTMLDDA]
---*/

var IsHTMLDDA = $262.IsHTMLDDA;

assert.sameValue(
  (function() {
    switch (IsHTMLDDA) {
      case undefined: return 1;
      case null: return 2;
      case IsHTMLDDA: return 3;
    }
  })(),
  3
);
