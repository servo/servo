// Copyright (C) 2017 Valerie Young. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: ASI test in field declarations -- computed name interpreted as string index
esid: sec-automatic-semicolon-insertion
features: [class, class-fields-public]
---*/

var C = class {
  x = "lol"
  [1]
}

var c = new C();

assert.sameValue(c.x, 'o');
