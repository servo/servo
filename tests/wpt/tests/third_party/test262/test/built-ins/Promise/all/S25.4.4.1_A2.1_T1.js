// Copyright 2014 Cubane Canada, Inc.  All rights reserved.
// See LICENSE for details.

/*---
info: Promise.all([]) is a Promise
es6id: 25.4.4.1_A2.1_T1
author: Sam Mikes
description: Promise.all returns a Promise
---*/

var p = Promise.all([]);
assert(!!(p instanceof Promise), 'The value of !!(p instanceof Promise) is expected to be true');
