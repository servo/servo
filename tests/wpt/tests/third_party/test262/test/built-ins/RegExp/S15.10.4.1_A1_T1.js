// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    If pattern is an object R whose [[Class]] property is "RegExp" and flags is undefined, then let P be
    the pattern used to construct R and let F be the flags used to construct R
es5id: 15.10.4.1_A1_T1
description: Pattern is /./i and RegExp is new RegExp(pattern)
---*/

var __pattern = /./i;
var __re = new RegExp(__pattern);

assert.sameValue(
  __re.source,
  __pattern.source,
  'The value of __re.source is expected to equal the value of __pattern.source'
);

assert.sameValue(
  __re.multiline,
  __pattern.multiline,
  'The value of __re.multiline is expected to equal the value of __pattern.multiline'
);

assert.sameValue(
  __re.global,
  __pattern.global,
  'The value of __re.global is expected to equal the value of __pattern.global'
);

assert.sameValue(
  __re.ignoreCase,
  __pattern.ignoreCase,
  'The value of __re.ignoreCase is expected to equal the value of __pattern.ignoreCase'
);
