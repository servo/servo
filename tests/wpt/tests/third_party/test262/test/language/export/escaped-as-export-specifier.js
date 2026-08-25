// Copyright (C) 2017 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-grammar-notation
description: >
  The `as` contextual keyword must not contain Unicode escape sequences.
info: |
  Terminal symbols are shown
  in fixed width font, both in the productions of the grammars and throughout this
  specification whenever the text directly refers to such a terminal symbol. These
  are to appear in a script exactly as written. All terminal symbol code points
  specified in this way are to be understood as the appropriate Unicode code points
  from the Basic Latin range, as opposed to any similar-looking code points from
  other Unicode ranges.
negative:
  phase: parse
  type: SyntaxError
flags: [module]
---*/

$DONOTEVALUATE();

export var a = 0;
export {a \u0061s b} from "./escaped-as-export-specifier.js";
