// Copyright (C) 2024 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: Regular expression modifiers should not parse without the colon.
esid: sec-patterns-static-semantics-early-errors
features: [regexp-modifiers]
negative:
  phase: parse
  type: SyntaxError
info: |
    Atom :: ( ? RegularExpressionFlags - RegularExpressionFlags : Disjunction )
    ...
---*/

$DONOTEVALUATE();

/(?i-)/;
