// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: let F be the empty string if flags is undefined
es5id: 15.10.4.1_A4_T4
description: RegExp is new RegExp(null,void 0)
---*/

var __re = new RegExp(null, void 0);

assert.sameValue(__re.source, "null", 'The value of __re.source is expected to be "null"');
assert.sameValue(__re.multiline, false, 'The value of __re.multiline is expected to be false');
assert.sameValue(__re.global, false, 'The value of __re.global is expected to be false');
assert.sameValue(__re.ignoreCase, false, 'The value of __re.ignoreCase is expected to be false');
