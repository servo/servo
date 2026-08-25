// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    The production CharacterClass :: [ ^ ClassRanges ] evaluates by
    evaluating ClassRanges to  obtain a CharSet and returning that CharSet
    and the boolean true
es5id: 15.10.2.13_A2_T6
description: Execute /a[^b]c/.test("abc") and check results
---*/

var __executed = /a[^b]c/.test("abc");

assert(!__executed, 'The value of !__executed is expected to be true');
