// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    RegExp.prototype.exec(string) Performs a regular expression match of ToString(string) against the regular expression and
    returns an Array object containing the results of the match, or null if the string did not match
es5id: 15.10.6.2_A1_T16
description: RegExp is /undefined/ and call exec() without arguments
---*/

var __re = /undefined/.exec()[0];
assert.sameValue(__re, "undefined", 'The value of __re is expected to be "undefined"');
