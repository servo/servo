// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: "IdentifierStart :: _"
es5id: 7.6_A1.3_T1
description: Create variable _
---*/

//CHECK#1
var _ = 1;

assert.sameValue(_, 1);
