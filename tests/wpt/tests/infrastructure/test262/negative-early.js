/*---
negative:
  type: SyntaxError
  phase: early
---*/

"use strict";
$DONOTEVALUATE();
var let = 1; // 'let' is a restricted identifier in strict mode
