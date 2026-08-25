// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    RegExp.prototype.test behavior depends on the lastIndex property:
    ToLength(lastIndex) is the starting point for the search, so
    negative numbers result in searching from 0.
es5id: 15.10.6.3_A1_T22
description: "Set lastIndex to -1 and call /(?:ab|cd)\\d?/g.test(\"aacd22 \")"
---*/

var __re = /(?:ab|cd)\d?/g;
__re.lastIndex=-1;
var __executed = __re.test("aacd22 ");

assert(!!__executed, 'The value of !!__executed is expected to be true');
assert.sameValue(__re.lastIndex, 5, 'The value of __re.lastIndex is expected to be 5');

__re.lastIndex=-100;
__executed = __re.test("aacd22 ");

assert(!!__executed, 'The value of !!__executed is expected to be true');
assert.sameValue(__re.lastIndex, 5, 'The value of __re.lastIndex is expected to be 5');
