// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    Labelled statements are only used in conjunction with labelled
    break and continue statements
es5id: 12.12_A1_T1
description: Checking if labelled break works. See continue and break sections
---*/

var object = {p1: 1, p2: 1};
var result = 0;
lbl: for(var i in object){
  result += object[i];
  break lbl;
}

if(!(result === 1)){
  throw new Test262Error("'break label' should break execution of labelled iteration statement");
}
