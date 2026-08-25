// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: Assignment to string literals calls String constructor
es5id: 8.4_A9_T2
description: >
    Compare empty string variable, object String('') and object
    String()
---*/

var str="";
var strObj=new String("");
var strObj_=new String();

////////////////////////////////////////////////////////////
// CHECK#1
if (str.constructor !== strObj.constructor){
  throw new Test262Error('#1: "".constructor === new String("").constructor');
}
//
/////////////////////////////////////////////////////////////

////////////////////////////////////////////////////////////
// CHECK#2
if (str.constructor !== strObj_.constructor){
  throw new Test262Error('#2: "".constructor === new String().constructor');
}
//
/////////////////////////////////////////////////////////////

////////////////////////////////////////////////////////////
// CHECK#3
if (str != strObj){
  throw new Test262Error('#3: values of str=""; and strObj=new String(""); are equal');
}
//
/////////////////////////////////////////////////////////////

////////////////////////////////////////////////////////////
// CHECK#4
if (str === strObj){
  throw new Test262Error('#4: objects of str=""; and strObj=new String(""); are different');
}
//
/////////////////////////////////////////////////////////////

////////////////////////////////////////////////////////////
// CHECK#5
if (str != strObj_){
  throw new Test262Error('#5: values of str=""; and strObj=new String(); are equal');
}
//
/////////////////////////////////////////////////////////////

////////////////////////////////////////////////////////////
// CHECK#6
if (str === strObj_){
  throw new Test262Error('#6: objects of str=""; and strObj=new String(); are different');
}
//
/////////////////////////////////////////////////////////////
