// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: Syntax constructions of switch statement
es5id: 12.11_A3_T3
description: Checking if execution of "switch(value)" fails
negative:
  phase: parse
  type: SyntaxError
---*/

$DONOTEVALUATE();

switch(value);
