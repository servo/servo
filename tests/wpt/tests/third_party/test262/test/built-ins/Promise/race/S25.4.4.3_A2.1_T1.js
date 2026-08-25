// Copyright 2014 Cubane Canada, Inc.  All rights reserved.
// See LICENSE for details.

/*---
info: Promise.race returns a new promise
es6id: S25.4.4.3_A2.1_T1
author: Sam Mikes
description: Promise.race returns a new promise
---*/

var p = Promise.race([]);

assert(!!(p instanceof Promise), 'The value of !!(p instanceof Promise) is expected to be true');
