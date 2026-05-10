// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    The RegExp.prototype ignoreCase property does not have the attribute
    DontDelete
es5id: 15.10.7.3_A9
description: Checking if deleting the ignoreCase property succeeds
---*/

var __re = RegExp.prototype;

assert.sameValue(__re.hasOwnProperty('ignoreCase'), true, '__re.hasOwnProperty(\'ignoreCase\') must return true');
assert.sameValue(delete __re.ignoreCase, true, 'The value of `delete __re.ignoreCase` is expected to be true');
assert.sameValue(__re.hasOwnProperty('ignoreCase'), false, '__re.hasOwnProperty(\'ignoreCase\') must return false');

// TODO: Convert to verifyProperty() format.
