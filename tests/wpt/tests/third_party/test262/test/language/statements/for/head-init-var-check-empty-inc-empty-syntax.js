// Copyright 2019 Mike Pennisi.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: For head may omit optional expressions
es5id: 12.6.3_A9
---*/

var supreme=5;

for(var count=0;;) {if (count===supreme)break;else count++; }
