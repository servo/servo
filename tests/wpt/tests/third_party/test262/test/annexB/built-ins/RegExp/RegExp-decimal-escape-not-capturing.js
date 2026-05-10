// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    An escape sequence of the form \ followed by a nonzero decimal number n
    matches the result of the nth set of capturing parentheses (see
    15.10.2.11)
es5id: 15.10.2.9_A1_T4
es6id: B.1.4
description: >
    Execute /\b(\w+) \2\b/.test("do you listen the the band") and
    check results
---*/

var executed = /\b(\w+) \2\b/.test("do you listen the the band");

assert.sameValue(executed, false);
