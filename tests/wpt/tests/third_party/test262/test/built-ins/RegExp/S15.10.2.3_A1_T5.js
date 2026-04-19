// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    The | regular expression operator separates two alternatives.
    The pattern first tries to match the left Alternative (followed by the sequel of the regular expression).
    If it fails, it tries to match the right Disjunction (followed by the sequel of the regular expression)
es5id: 15.10.2.3_A1_T5
description: >
    Execute /\d{3}|[a-z]{4}/.test("2, 12 and 23 AND 0.00.1") and check
    results
---*/

var __executed = /\d{3}|[a-z]{4}/.test("2, 12 and 23 AND 0.00.1");

assert(!__executed, 'The value of !__executed is expected to be true');
