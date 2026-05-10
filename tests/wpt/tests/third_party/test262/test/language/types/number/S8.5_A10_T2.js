// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: Infinity is not a keyword
es5id: 8.5_A10
description: Create variable entitled Infinity
flags: [noStrict]
---*/

var Infinity=1.0;
Infinity='asdf';
Infinity=true;
