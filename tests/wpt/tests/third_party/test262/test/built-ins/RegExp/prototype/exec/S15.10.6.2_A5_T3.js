// Copyright 2015 the V8 project authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    RegExp.prototype.exec behavior depends on the lastIndex property:
    ToLength(lastIndex) is the starting point for the search, so
    negative numbers result in searching from 0.
es5id: 15.10.6.2_A5_T3
description: "Set lastIndex to -1 and call /(?:ab|cd)\\d?/g.exec(\"aacd22 \")"
---*/

var __re = /(?:ab|cd)\d?/g;
__re.lastIndex=-1;
var __executed = __re.exec("aacd22 ");

assert.sameValue(__executed[0], "cd2", 'The value of __executed[0] is expected to be "cd2"');
assert.sameValue(__re.lastIndex, 5, 'The value of __re.lastIndex is expected to be 5');

__re.lastIndex=-100;
__executed = __re.exec("aacd22 ");

assert.sameValue(__executed[0], "cd2", 'The value of __executed[0] is expected to be "cd2"');
assert.sameValue(__re.lastIndex, 5, 'The value of __re.lastIndex is expected to be 5');
