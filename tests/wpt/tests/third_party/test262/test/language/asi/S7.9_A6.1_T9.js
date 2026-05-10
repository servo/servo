// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: Check For Statement for automatic semicolon insertion
es5id: 7.9_A6.1_T9
description: for (false \n two semicolons \n)
---*/

//CHECK#1
for(false
    ;;
) {
  break;
}
