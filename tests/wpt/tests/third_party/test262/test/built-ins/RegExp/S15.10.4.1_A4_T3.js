// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: let F be the empty string if flags is undefined
es5id: 15.10.4.1_A4_T3
description: Use undefined properties of object as flags of RegExp
---*/

var __re = new RegExp({}.p, {}.q);

assert.sameValue(__re.multiline, false, 'The value of __re.multiline is expected to be false');
assert.sameValue(__re.global, false, 'The value of __re.global is expected to be false');
assert.sameValue(__re.ignoreCase, false, 'The value of __re.ignoreCase is expected to be false');
