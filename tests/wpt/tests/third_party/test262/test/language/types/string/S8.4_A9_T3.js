// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: Assignment to string literals calls String constructor
es5id: 8.4_A9_T3
description: Simple empty string variable compare with empty object String
---*/

var str="";
var strObj=new String;

////////////////////////////////////////////////////////////
// CHECK#1
if (str.constructor !== strObj.constructor){
  throw new Test262Error('#1: "".constructor === new String.constructor');
}
//
/////////////////////////////////////////////////////////////

////////////////////////////////////////////////////////////
// CHECK#2
if (str != strObj){
  throw new Test262Error('#2: values of str=""; and strObj=new String(""); are equal');
}
//
/////////////////////////////////////////////////////////////

////////////////////////////////////////////////////////////
// CHECK#3
if (str === strObj){
  throw new Test262Error('#3: objects of str=""; and strObj=new String(""); are different');
}
//
/////////////////////////////////////////////////////////////

//////////////////////////////////////////////////////////////////////////////
//CHECK#4
if (typeof str == typeof strObj){
  throw new Test262Error('#4: types of str=""; and strObj=new String(""); are different');
}
//
//////////////////////////////////////////////////////////////////////////////
