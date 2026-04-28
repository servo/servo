// Copyright 2019 Mike Pennisi.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: For head may omit optional expressions
es5id: 12.6.3_A9.1
---*/


var supreme = 5
var count;

for(count=0;;) {if (count===supreme)break;else count++; }
