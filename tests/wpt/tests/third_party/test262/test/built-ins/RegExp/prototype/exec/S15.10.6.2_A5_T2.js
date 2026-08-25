// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    RegExp.prototype.exec behavior depends on global property.
    Let global is true and let I = If ToInteger(lastIndex).
    Then if I<0 orI>length then set lastIndex to 0 and return null
es5id: 15.10.6.2_A5_T2
description: "Set lastIndex to 100 and call /(?:ab|cd)\\d?/g.exec(\"aacd22 \")"
---*/

var __re = /(?:ab|cd)\d?/g;
__re.lastIndex=100;
var __executed = __re.exec("aacd22 ");

assert(!__executed, 'The value of !__executed is expected to be true');
assert.sameValue(__re.lastIndex, 0, 'The value of __re.lastIndex is expected to be 0');
