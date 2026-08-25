// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    The production Assertion :: \b evaluates by returning an internal
    AssertionTester closure that takes a State argument x and performs the ...
es5id: 15.10.2.6_A3_T13
description: >
    Execute /op\b/.test("pilot\nsoviet robot\topenoffice") and check
    results
---*/

var __executed = /op\b/.test("pilot\nsoviet robot\topenoffice");

assert(!__executed, 'The value of !__executed is expected to be true');
