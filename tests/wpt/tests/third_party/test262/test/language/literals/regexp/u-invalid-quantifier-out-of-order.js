// Copyright (C) 2026 hexbinoct. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-patterns-static-semantics-early-errors
description: >
    Braced quantifier with a lower bound greater than its upper bound (`u` flag)
info: |
    QuantifierPrefix :: { DecimalDigits , DecimalDigits }

    It is a Syntax Error if the MV of the first DecimalDigits is strictly
    greater than the MV of the second DecimalDigits.

    The Annex B pattern grammar does not change the syntax of patterns parsed
    with the [UnicodeMode] parameter, so this rule applies unmodified.
negative:
  phase: parse
  type: SyntaxError
---*/

$DONOTEVALUATE();

/a{2,1}/u;
