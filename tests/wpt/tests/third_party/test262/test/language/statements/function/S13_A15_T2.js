// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: "''arguments'' variable overrides ActivationObject.arguments"
es5id: 13_A15_T2
description: Overriding arguments within functions body
flags: [noStrict]
---*/

THE_ANSWER="Answer to Life, the Universe, and Everything";

function __func(){
    var arguments = THE_ANSWER;
    return arguments;
};

//////////////////////////////////////////////////////////////////////////////
//CHECK#1
if (__func(42,42,42) !== THE_ANSWER) {
	throw new Test262Error('#1:  "arguments" variable overrides ActivationObject.arguments');
}
//
//////////////////////////////////////////////////////////////////////////////
