// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: Multi line comments cannot nest
es5id: 7.4_A3
description: Try use nested comments
negative:
  phase: parse
  type: SyntaxError
---*/

$DONOTEVALUATE();

/*CHECK#1*/

/*
var

/* x */
= 1;
*/
