// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: Check For Statement for automatic semicolon insertion
es5id: 7.9_A6.1_T8
description: for (false semicolon false \n semicolon false \n)
---*/

//CHECK#1
for(false;false
  ;false
) {
  break;
}
