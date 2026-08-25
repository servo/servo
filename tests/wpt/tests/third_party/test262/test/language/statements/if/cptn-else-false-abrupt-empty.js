// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-if-statement-runtime-semantics-evaluation
description: >
    Completion value when expression is false with an `else` clause and body
    returns an empty abrupt completion
info: |
    IfStatement : if ( Expression ) Statement else Statement

    3. If exprValue is true, then
       [...]
    4. Else,
       a. Let stmtCompletion be the result of evaluating the second Statement.
    5. Return Completion(UpdateEmpty(stmtCompletion, undefined)).
---*/

assert.sameValue(
  eval('1; do { if (false) { } else { break; } } while (false)'), undefined
);
assert.sameValue(
  eval('2; do { 3; if (false) { } else { break; } } while (false)'), undefined
);
assert.sameValue(
  eval('4; do { if (false) { 5; } else { break; } } while (false)'), undefined
);
assert.sameValue(
  eval('6; do { 7; if (false) { 8; } else { break; } } while (false)'),
  undefined
);

assert.sameValue(
  eval('9; do { if (false) { } else { continue; } } while (false)'), undefined
);
assert.sameValue(
  eval('10; do { 11; if (false) { } else { continue; } } while (false)'),
  undefined
);
assert.sameValue(
  eval('12; do { if (false) { 13; } else { continue; } } while (false)'),
  undefined
);
assert.sameValue(
  eval('14; do { 15; if (false) { 16; } else { continue; } } while (false)'),
  undefined
);
