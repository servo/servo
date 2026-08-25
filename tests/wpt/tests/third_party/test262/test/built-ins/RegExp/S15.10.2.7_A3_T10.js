// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    The production QuantifierPrefix :: + evaluates by returning the two
    results 1 and \infty
es5id: 15.10.2.7_A3_T10
description: Execute /o+/.test("abcdefg") and check results
---*/

var __executed = /o+/.test("abcdefg");

assert(!__executed, 'The value of !__executed is expected to be true');
