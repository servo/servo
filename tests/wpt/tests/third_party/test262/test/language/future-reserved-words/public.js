// Copyright 2011 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-identifiers-static-semantics-early-errors
info: The "public" token can be used as identifier in non-strict code
es5id: 7.6.1.2_A1.24ns
description: Checking if execution of "public=1" succeeds in non-strict code
flags: [noStrict]
---*/

var public = 1;
var publi\u0063 = 2;

{ let public = 3; }
{ let publi\u0063 = 4; }

{ const public = 5; }
{ const publi\u0063 = 6; }
