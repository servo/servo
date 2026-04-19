// Copyright (C) 2017 Valerie Young. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: ASI test in field declarations -- computed name interpreted as property
esid: sec-automatic-semicolon-insertion
features: [class, class-fields-public]
---*/

var obj = {}
class C {
  x = obj
  ['lol'] = 42
}

var c = new C();

assert.sameValue(c.x, 42);
assert.sameValue(obj['lol'], 42);
