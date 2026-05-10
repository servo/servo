// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 13.6.7
description: Completion value when expression is false without an `else` clause
info: |
    IfStatement : if ( Expression ) Statement

    [...]
    4. If exprValue is false, then
       a. Return NormalCompletion(undefined).
---*/

assert.sameValue(eval('1; if (false) { }'), undefined);
assert.sameValue(eval('2; if (false) { 3; }'), undefined);
