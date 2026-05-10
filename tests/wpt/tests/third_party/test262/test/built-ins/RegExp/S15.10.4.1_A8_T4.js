// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: let P be ToString(pattern) and let F be ToString(flags)
es5id: 15.10.4.1_A8_T4
description: >
    Pattern is {toString:void 0,valueOf:function(){return "[z-z]";}}
    and flags is {toString:void 0,valueOf:function(){return "mig";}}
---*/

var __re = new RegExp({toString:void 0,valueOf:function(){return "[z-z]";}}, {toString:void 0,valueOf:function(){return "mig";}});

assert.sameValue(__re.ignoreCase, true, 'The value of __re.ignoreCase is expected to be true');
assert.sameValue(__re.multiline, true, 'The value of __re.multiline is expected to be true');
assert.sameValue(__re.global, true, 'The value of __re.global is expected to be true');
assert.sameValue(__re.lastIndex, 0, 'The value of __re.lastIndex is expected to be 0');
assert.notSameValue(typeof __re.source, "undefined", 'The value of typeof __re.source is not "undefined"');
