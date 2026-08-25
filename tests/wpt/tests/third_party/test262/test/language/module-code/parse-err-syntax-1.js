// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
description: Early SyntaxError resulting from module parsing
esid: sec-parsemodule
negative:
  phase: parse
  type: SyntaxError
info: |
    [...]
    2. Parse sourceText using Module as the goal symbol and analyze the parse
       result for any Early Error conditions. If the parse was successful and
       no early errors were found, let body be the resulting parse tree.
       Otherwise, let body be a List of one or more SyntaxError objects
       representing the parsing errors and/or early errors.
flags: [module]
---*/

$DONOTEVALUATE();

?
