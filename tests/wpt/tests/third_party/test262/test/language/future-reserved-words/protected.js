// Copyright 2011 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-identifiers-static-semantics-early-errors
info: The "protected" token can be used as identifier in non-strict code
es5id: 7.6.1.2_A1.23ns
description: Checking if execution of "protected=1" succeeds in non-strict code
flags: [noStrict]
---*/

var protected = 1;
var prot\u0065cted = 2;

{ let protected = 3; }
{ let prot\u0065cted = 4; }

{ const protected = 5; }
{ const prot\u0065cted = 6; }
