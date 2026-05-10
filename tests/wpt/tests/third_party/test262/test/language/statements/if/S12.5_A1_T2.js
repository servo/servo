// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: 1, true, non-empty string in expression is evaluated to true
es5id: 12.5_A1_T2
description: Using "if/else" construction
---*/

var c=0;
//////////////////////////////////////////////////////////////////////////////
//CHECK#1
if(!(1))
	throw new Test262Error('#1.1: 1 in expression is evaluated to true');
else
  c++;
if (c!=1) throw new Test262Error('#1.2: else branch don`t execute');
//
//////////////////////////////////////////////////////////////////////////////

//////////////////////////////////////////////////////////////////////////////
//CHECK#2
if(!(true))
	throw new Test262Error('#2.1: true in expression is evaluated to true');
else
  c++;
if (c!=2) throw new Test262Error('#2.2: else branch don`t execute');
//
//////////////////////////////////////////////////////////////////////////////

//////////////////////////////////////////////////////////////////////////////
//CHECK#3
if(!("1"))
	throw new Test262Error('#3.1: "1" in expression is evaluated to true');
else
  c++;
if (c!=3) throw new Test262Error('#3.2: else branch don`t execute');
//
//////////////////////////////////////////////////////////////////////////////

//////////////////////////////////////////////////////////////////////////////
//CHECK#4
if(!("A"))
	throw new Test262Error('#4.1: "A" in expression is evaluated to true');
else
  c++;
if (c!=4) throw new Test262Error('#4.2: else branch don`t execute');
//
//////////////////////////////////////////////////////////////////////////////
