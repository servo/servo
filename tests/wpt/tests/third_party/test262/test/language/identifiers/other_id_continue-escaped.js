// Copyright (C) 2017 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-names-and-keywords
description: Test grandfathered characters of ID_Continue.
info: |
  Grandfathered characters (Other_ID_Start + Other_ID_Continue)
---*/

// Other_ID_Start (Unicode 4.0)
var a\u2118;
var a\u212E;
var a\u309B;
var a\u309C;

// Other_ID_Start (Unicode 9.0)
var a\u1885;
var a\u1886;

// Other_ID_Continue (Unicode 4.1)
var a\u1369;
var a\u136A;
var a\u136B;
var a\u136C;
var a\u136D;
var a\u136E;
var a\u136F;
var a\u1370;
var a\u1371;

// Other_ID_Continue (Unicode 5.1)
var a\u00B7;
var a\u0387;

// Other_ID_Continue (Unicode 6.0)
var a\u19DA;
