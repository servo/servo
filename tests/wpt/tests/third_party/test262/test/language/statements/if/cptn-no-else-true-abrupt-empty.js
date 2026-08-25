// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-if-statement-runtime-semantics-evaluation
description: >
    Completion value when expression is true without an `else` clause and body
    returns an empty abrupt completion
info: |
    IfStatement : if ( Expression ) Statement

    3. If exprValue is false, then
       [...]
    4. Else,
       a. Let stmtCompletion be the result of evaluating Statement.
       b. Return Completion(UpdateEmpty(stmtCompletion, undefined)).
---*/

assert.sameValue(
  eval('1; do { 2; if (true) { 3; break; } 4; } while (false)'), 3
);
assert.sameValue(
  eval('5; do { 6; if (true) { break; } 7; } while (false)'), undefined
);

assert.sameValue(
  eval('8; do { 9; if (true) { 10; continue; } 11; } while (false)'), 10
);
assert.sameValue(
  eval('12; do { 13; if (true) { continue; } 14; } while (false)'), undefined
);
