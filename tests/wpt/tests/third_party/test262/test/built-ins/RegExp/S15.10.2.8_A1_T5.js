// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    The form (?= Disjunction ) specifies a zero-width positive lookahead.
    In order for it to succeed, the pattern inside Disjunction must match at the current position, but the current position is not advanced before matching the sequel.
    If Disjunction can match at the current position in several ways, only the first one is tried
es5id: 15.10.2.8_A1_T5
description: >
    Execute /[Jj]ava([Ss]cript)?(?=\:)/.test("rhino is JavaScript
    engine") and check results
---*/

var __executed = /[Jj]ava([Ss]cript)?(?=\:)/.test("rhino is JavaScript engine");

assert(!__executed, 'The value of !__executed is expected to be true');
