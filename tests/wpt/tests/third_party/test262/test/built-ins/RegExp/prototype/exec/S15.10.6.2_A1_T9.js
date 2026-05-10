// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    RegExp.prototype.exec(string) Performs a regular expression match of ToString(string) against the regular expression and
    returns an Array object containing the results of the match, or null if the string did not match
es5id: 15.10.6.2_A1_T9
description: String is undefined variable and RegExp is /1|12/
---*/

var __string;

var __re = /1|12/;
assert.sameValue(__re.exec(__string), null, '__re.exec() must return null');

function __string(){}
