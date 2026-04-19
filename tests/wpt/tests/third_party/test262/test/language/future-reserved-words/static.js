// Copyright 2011 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-identifiers-static-semantics-early-errors
info: The "static" token can be used as identifier in non-strict code
es5id: 7.6.1.2_A1.26ns
description: Checking if execution of "static=1" succeeds in non-strict code
flags: [noStrict]
---*/

var static = 1;
var st\u0061tic = 2;

{ let static = 3; }
{ let st\u0061tic = 4; }

{ const static = 5; }
{ const st\u0061tic = 6; }
