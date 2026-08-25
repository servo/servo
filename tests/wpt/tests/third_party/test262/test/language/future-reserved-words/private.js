// Copyright 2011 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-identifiers-static-semantics-early-errors
info: The "private" token can be used as identifier in non-strict code
es5id: 7.6.1.2_A1.22ns
description: Checking if execution of "private=1" succeeds in non-strict code
flags: [noStrict]
---*/

var private = 1;
var priv\u0061te = 2;

{ let private = 3; }
{ let priv\u0061te = 4; }

{ const private = 5; }
{ const priv\u0061te = 6; }
