// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    When String.prototype.charCodeAt(pos) calls if ToInteger(pos) not less
    than ToString(this value) the NaN returns
es5id: 15.5.4.5_A3
description: pos is bigger of string length
---*/

var __instance = new String("ABC");

assert.sameValue(__instance.charCodeAt(3), NaN);
