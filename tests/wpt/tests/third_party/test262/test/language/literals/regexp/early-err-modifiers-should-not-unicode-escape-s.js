// Copyright (C) 2024 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: Source text cannot use unicode escape sequences to express code points "i", "m", "s" (regular expression flags)
esid: sec-patterns-static-semantics-early-errors
features: [regexp-modifiers]
negative:
  phase: parse
  type: SyntaxError
info: |
    Atom :: ( ? RegularExpresisonFlags : Disjunction )
    It is a Syntax Error if the source text matched by RegularExpressionFlags contains any code points other than "i", "m", "s", or if it contains the same code point more than once.

---*/

$DONOTEVALUATE();

/(?\u0073:a)/u /* s */;
