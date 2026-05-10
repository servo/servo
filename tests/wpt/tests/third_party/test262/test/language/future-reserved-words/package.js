// Copyright 2011 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-identifiers-static-semantics-early-errors
info: The "package" token can be used as identifier in non-strict code
es5id: 7.6.1.2_A1.21ns
description: Checking if execution of "package=1" succeeds in non-strict code
flags: [noStrict]
---*/

var package = 1;
var pack\u0061ge = 2;

{ let package = 3; }
{ let pack\u0061ge = 4; }

{ const package = 5; }
{ const pack\u0061ge = 6; }
