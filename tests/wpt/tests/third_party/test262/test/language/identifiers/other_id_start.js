// Copyright (C) 2017 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-names-and-keywords
description: Test grandfathered characters of ID_Start.
info: |
  Grandfathered characters (Other_ID_Start)
---*/

// Other_ID_Start (Unicode 4.0)
var ℘; // U+2118
var ℮; // U+212E
var ゛; // U+309B
var ゜; // U+309C

// Other_ID_Start (Unicode 9.0)
var ᢅ; // U+1885
var ᢆ; // U+1886
