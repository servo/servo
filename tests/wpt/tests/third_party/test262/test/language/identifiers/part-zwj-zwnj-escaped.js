// Copyright (C) 2020 Adam Kluball. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
author: Adam Kluball
esid: sec-names-and-keywords
description: zero width joiner and zero width non-joiner are valid identifier parts
---*/

var $ = 1;
var $\u200D = 2;
var $\u200C = 3;

assert.sameValue($, 1);
assert.sameValue($\u200D, 2);
assert.sameValue($\u200C, 3);
