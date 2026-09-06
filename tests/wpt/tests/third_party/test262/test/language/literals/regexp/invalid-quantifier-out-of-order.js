// Copyright (C) 2026 hexbinoct. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-patterns-static-semantics-early-errors
description: >
    Braced quantifier with a lower bound greater than its upper bound
info: |
    QuantifierPrefix :: { DecimalDigits , DecimalDigits }

    It is a Syntax Error if the MV of the first DecimalDigits is strictly
    greater than the MV of the second DecimalDigits.

    Annex B replaces Term with an alternative that admits ExtendedAtom, but
    B.1.2.1 does not modify this rule, and the ExtendedAtom Quantifier
    alternative is considered before the bare ExtendedAtom alternative. The
    SyntaxError is therefore consistent between Annex-B and non-Annex-B
    environments.
negative:
  phase: parse
  type: SyntaxError
---*/

$DONOTEVALUATE();

/a{2,1}/;
