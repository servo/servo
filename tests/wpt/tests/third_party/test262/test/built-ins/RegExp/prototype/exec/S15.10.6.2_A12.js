// Copyright 2011 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: regExp exec() acts like regExp.exec('undefined') (step 2)
es5id: 15.10.6.2_A12
description: Checking RegExp.prototype.exec
---*/

(/foo/).test('xfoox');
var match = new RegExp('(.|\r|\n)*','').exec()[0];
assert.notSameValue(match, 'xfoox', 'The value of match is not "xfoox"');
assert.sameValue(match, 'undefined', 'The value of match is expected to be "undefined"');
