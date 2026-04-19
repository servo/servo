// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    The RegExp.prototype global property does not have the attribute
    DontDelete
es5id: 15.10.7.2_A9
description: Checking if deleting the global property succeeds
---*/

var __re = RegExp.prototype;

assert.sameValue(__re.hasOwnProperty('global'), true, '__re.hasOwnProperty(\'global\') must return true');
assert.sameValue(delete __re.global, true, 'The value of `delete __re.global` is expected to be true');
assert.sameValue(__re.hasOwnProperty('global'), false, '__re.hasOwnProperty(\'global\') must return false');

// TODO: Convert to verifyProperty() format.
