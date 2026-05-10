// Copyright (C) 2016 Jordan Harband. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-string.prototype.padend
description: >
  String#padEnd should return the string unchanged when
  an explicit empty string is provided
author: Jordan Harband
---*/

assert.sameValue('abc'.padEnd(5, ''), 'abc');
