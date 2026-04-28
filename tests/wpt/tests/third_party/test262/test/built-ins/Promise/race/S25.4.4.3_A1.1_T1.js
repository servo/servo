// Copyright 2014 Cubane Canada, Inc.  All rights reserved.
// See LICENSE for details.

/*---
info: Promise.race is callable
es6id: S25.4.4.3_A1.1_T1
author: Sam Mikes
description: Promise.race is callable
---*/
assert.sameValue(typeof Promise.race, "function", 'The value of `typeof Promise.race` is expected to be "function"');
