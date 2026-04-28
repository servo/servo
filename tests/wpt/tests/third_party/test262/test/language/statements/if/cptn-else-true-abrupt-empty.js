// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-if-statement-runtime-semantics-evaluation
description: >
    Completion value when expression is true with an `else` clause and body
    returns an abrupt completion
info: |
    IfStatement : if ( Expression ) Statement else Statement

    3. If exprValue is true, then
       a. Let stmtCompletion be the result of evaluating the first Statement.
    4. Else,
       [...]
    5. Return Completion(UpdateEmpty(stmtCompletion, undefined)).
---*/

assert.sameValue(
  eval('1; do { if (true) { break; } else { } } while (false)'), undefined
);
assert.sameValue(
  eval('2; do { 3; if (true) { break; } else { } } while (false)'), undefined
);
assert.sameValue(
  eval('4; do { if (true) { break; } else { 5; } } while (false)'), undefined
);
assert.sameValue(
  eval('6; do { 7; if (true) { break; } else { 8; } } while (false)'),
  undefined
);

assert.sameValue(
  eval('1; do { if (true) { continue; } else { } } while (false)'), undefined
);
assert.sameValue(
  eval('2; do { 3; if (true) { continue; } else { } } while (false)'), undefined
);
assert.sameValue(
  eval('4; do { if (true) { continue; } else { 5; } } while (false)'), undefined
);
assert.sameValue(
  eval('6; do { 7; if (true) { continue; } else { 8; } } while (false)'),
  undefined
);
