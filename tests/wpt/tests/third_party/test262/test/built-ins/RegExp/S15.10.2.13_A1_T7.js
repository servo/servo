// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    The production CharacterClass :: [ [lookahead \notin {^}] ClassRanges ]
    evaluates by evaluating ClassRanges to obtain a CharSet and returning
    that CharSet and the boolean false
es5id: 15.10.2.13_A1_T7
description: Execute /ab[erst]de/.test("abcde") and check results
---*/

var __executed = /ab[erst]de/.test("abcde");

assert(!__executed, 'The value of !__executed is expected to be true');
