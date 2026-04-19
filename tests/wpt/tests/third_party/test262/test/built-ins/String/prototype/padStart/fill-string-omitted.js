// Copyright (C) 2016 Jordan Harband. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-string.prototype.padstart
description: String#padStart should default to a fillString of " " when omitted
author: Jordan Harband
---*/

assert.sameValue('abc'.padStart(5), '  abc');
assert.sameValue('abc'.padStart(5, undefined), '  abc');
