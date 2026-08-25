// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: An Object is an unordered collection of properties
es5id: 8.6_A4_T1
description: Simple using a few custom properties
---*/

///////////////////////////////////////////////////////
// CHECK#1
var obj = {bar:true, some:1, foo:"a"};

var count=0;

for (var property in obj)	count++;

if (count !== 3){
  throw new Test262Error('#1: obj = {bar:true, some:1, foo:"a"}; count=0; for (property in obj) count++; count === 3. Actual: ' + (count));
}
//
////////////////////////////////////////////////////////

///////////////////////////////////////////////////////
// CHECK#2
var obj_ = {bar:true};
obj_.some = 1;
obj_.foo = "a";

count=0;

for (property in obj_) count++;

if (count !== 3){
  throw new Test262Error('#2: obj_ = {bar:true}; obj_.some = 1; obj_.foo = "a"; count=0; for (property in obj_) count++; count === 3. Actual: ' + (count));
}
//
////////////////////////////////////////////////////////

///////////////////////////////////////////////////////
// CHECK#3
var obj__ = new Object();
obj__.bar = true;
obj__.some = 1;
obj__.foo = "a";

count=0;

for (property in obj__)	count++;

if (count !== 3){
  throw new Test262Error('#3: obj__ = new Object(); obj__.bar = true; obj__.some = 1; obj__.foo = "a"; for (property in obj__)  count++; count === 3. Actual: ' + (count));
}
//
////////////////////////////////////////////////////////
