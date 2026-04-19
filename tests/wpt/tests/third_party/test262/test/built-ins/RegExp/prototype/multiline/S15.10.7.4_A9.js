// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    The RegExp.prototype multiline property does not have the attribute
    DontDelete
es5id: 15.10.7.4_A9
description: Checking if deleting the multiline property succeeds
---*/

var __re = RegExp.prototype;

assert.sameValue(__re.hasOwnProperty('multiline'), true, '__re.hasOwnProperty(\'multiline\') must return true');
assert.sameValue(delete __re.multiline, true, 'The value of `delete __re.multiline` is expected to be true');
assert.sameValue(__re.hasOwnProperty('multiline'), false, '__re.hasOwnProperty(\'multiline\') must return false');

// TODO: Convert to verifyProperty() format.
