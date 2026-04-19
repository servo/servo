// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    The production CharacterClass :: [ [lookahead \notin {^}] ClassRanges ]
    evaluates by evaluating ClassRanges to obtain a CharSet and returning
    that CharSet and the boolean false
es5id: 15.10.2.13_A1_T1
description: Execute /[]a/.test("\0a\0a") and check results
---*/

var __executed = /[]a/.test("\0a\0a");
assert(!__executed, 'The value of !__executed is expected to be true');
