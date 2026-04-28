// Copyright (C) 2016 Jordan Harband. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-string.prototype.padend
description: String#padEnd should default to a fillString of " " when omitted
author: Jordan Harband
---*/

assert.sameValue('abc'.padEnd(5), 'abc  ');
assert.sameValue('abc'.padEnd(5, undefined), 'abc  ');
