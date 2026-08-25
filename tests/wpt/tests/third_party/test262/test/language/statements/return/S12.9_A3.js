// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: If Expression is omitted, the return value is undefined
es5id: 12.9_A3
description: Return without Expression
---*/

var __evaluated;
__evaluated = (function (){return;})();

//////////////////////////////////////////////////////////////////////////////
//CHECK#1
if (__evaluated !== undefined) {
	throw new Test262Error('#1: If Expression is omitted, the return value is undefined');
}
//
//////////////////////////////////////////////////////////////////////////////
