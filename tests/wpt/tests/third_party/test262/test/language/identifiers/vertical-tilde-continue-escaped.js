// Copyright (C) 2017 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-names-and-keywords
description: Test VERTICAL TILDE (U+2E2F) is not recognized as ID_Continue character.
info: |
  VERTICAL TILDE is in General Category 'Lm' and [:Pattern_Syntax:].
negative:
  phase: parse
  type: SyntaxError
---*/

$DONOTEVALUATE();

var a\u2E2F;
