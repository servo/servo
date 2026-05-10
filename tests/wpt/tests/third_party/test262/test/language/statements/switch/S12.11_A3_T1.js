// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: Syntax constructions of switch statement
es5id: 12.11_A3_T1
description: Checking if execution of "switch() {}" fails
negative:
  phase: parse
  type: SyntaxError
---*/

$DONOTEVALUATE();

function SwitchTest(value){
  var result = 0;

  switch() {
    case 0:
      result += 2;
    default:
      result += 32;
      break;
  }

  return result;
}

var x = SwitchTest(0);
