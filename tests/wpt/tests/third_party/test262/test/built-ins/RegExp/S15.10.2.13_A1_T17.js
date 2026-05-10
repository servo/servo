// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    The production CharacterClass :: [ [lookahead \notin {^}] ClassRanges ]
    evaluates by evaluating ClassRanges to obtain a CharSet and returning
    that CharSet and the boolean false
es5id: 15.10.2.13_A1_T17
description: Execute /[]/.exec("a[b\n[]\tc]d") and check results
---*/

var __executed = /[]/.exec("a[b\n[]\tc]d");

assert.sameValue(__executed, null, 'The value of __executed is expected to be null');
