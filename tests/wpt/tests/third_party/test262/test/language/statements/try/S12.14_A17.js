// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: Using "try" with "catch" or "finally" statement in a constructor
es5id: 12.14_A17
description: Creating exceptions within constructor
---*/

var i=1;
function Integer( value, exception ) {
  try{
    this.value = checkValue( value );
    if(exception) throw new Test262Error('#'+i+'.1: Must be exception');
  }
  catch(e){
    this.value = e.toString();
    if(!exception) throw new Test262Error('#'+i+'.2: Don`t must be exception');
  }
  i++;
}

function checkValue(value){
  if(Math.floor(value)!=value||isNaN(value)){
    throw (INVALID_INTEGER_VALUE +": " + value);
  }
  else{
    return value;
  }
}

// CHECK#1
new Integer(13, false);
// CHECK#2
new Integer(NaN, true);
// CHECK#3
new Integer(0, false);
// CHECK#4
new Integer(Infinity, false);
// CHECK#5
new Integer(-1.23, true);
// CHECK#6
new Integer(Math.LN2, true);
