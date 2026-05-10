// Copyright 2011 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-identifiers-static-semantics-early-errors
info: The "implements" token can be used as identifier in non-strict code
es5id: 7.6.1.2_A1.15ns
description: Checking if execution of "implements=1" succeeds in non-strict code
flags: [noStrict]
---*/

var implements = 1;
var impl\u0065ments = 2;

{ let implements = 3; }
{ let impl\u0065ments = 4; }

{ const implements = 5; }
{ const impl\u0065ments = 6; }
