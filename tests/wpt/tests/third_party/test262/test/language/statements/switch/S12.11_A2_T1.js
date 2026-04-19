// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: There can be only one DefaultClause
es5id: 12.11_A2_T1
description: Duplicate DefaultClause
negative:
  phase: parse
  type: SyntaxError
---*/

$DONOTEVALUATE();

function SwitchTest(value){
  var result = 0;

  switch(value) {
    case 0:
      result += 2;
    default:
      result += 32;
      break;
    default:
      result += 32;
      break;
  }

  return result;
}

var x = SwitchTest(0);
