// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.10.4.1-1
description: >
    RegExp - no TypeError is thrown when pattern is an object and
    has a [[RegExpMatcher]] internal slot, and flags is not undefined
---*/

var regObj = new RegExp();
var regExpObj = new RegExp(regObj, "g");
assert(regExpObj.global);
