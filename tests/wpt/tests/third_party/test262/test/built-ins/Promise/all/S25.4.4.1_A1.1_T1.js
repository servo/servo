// Copyright 2014 Cubane Canada, Inc.  All rights reserved.
// See LICENSE for details.

/*---
info: Promise.all is callable
es6id: 25.4.4.1_A1.1_T1
author: Sam Mikes
description: Promise.all is callable
---*/
assert.sameValue(typeof Promise.all, "function", 'The value of `typeof Promise.all` is expected to be "function"');
