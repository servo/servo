// Copyright 2018 Mathias Bynens. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
author: Mathias Bynens
description: >
  NonemptyClassRangesNoDash :: ClassAtomNoDash - ClassAtom ClassRanges

  It is a Syntax Error if IsCharacterClass of ClassAtomNoDash is true or
  IsCharacterClass of ClassAtom is true.
esid: sec-patterns-static-semantics-early-errors
negative:
  phase: parse
  type: SyntaxError
features: [regexp-unicode-property-escapes]
---*/

$DONOTEVALUATE();

/[\uFFFF-\p{Hex}]/u;
