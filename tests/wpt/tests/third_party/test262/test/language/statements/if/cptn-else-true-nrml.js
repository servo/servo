// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 13.6.7
description: >
    Completion value when expression is true with an `else` clause and body
    returns a normal completion
info: |
    IfStatement : if ( Expression ) Statement else Statement

    4. If exprValue is true, then
       a. Let stmtCompletion be the result of evaluating the first Statement.
    5. Else,
       [...]
    6. ReturnIfAbrupt(stmtCompletion).
    7. If stmtCompletion.[[value]] is not empty, return stmtCompletion.
    8. Return NormalCompletion(undefined).
---*/

assert.sameValue(eval('1; if (true) { } else { }'), undefined);
assert.sameValue(eval('2; if (true) { 3; } else { }'), 3);
assert.sameValue(eval('4; if (true) { } else { 5; }'), undefined);
assert.sameValue(eval('6; if (true) { 7; } else { 8; }'), 7);
