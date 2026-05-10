// Copyright 2018 Mathias Bynens. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
author: Mathias Bynens
description: >
  NonemptyClassRanges :: ClassAtom - ClassAtom ClassRanges

  It is a Syntax Error if IsCharacterClass of the first ClassAtom is
  true or IsCharacterClass of the second ClassAtom is true.
esid: sec-patterns-static-semantics-early-errors
negative:
  phase: parse
  type: SyntaxError
features: [regexp-unicode-property-escapes]
---*/

$DONOTEVALUATE();

/[--\p{Hex}]/u;
