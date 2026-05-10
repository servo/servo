// Copyright (C) 2019 Student Main. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
info: |
    IdentifierName and ReservedWord are tokens that are interpreted according to the 
    Default Identifier Syntax given in Unicode Standard Annex #31, 
    Identifier and Pattern Syntax, with some small modifications.
esid: sec-names-and-keywords
description: Check CJK UNIFIED IDEOGRAPH range is correct.
---*/

// CJK UNIFIED IDEOGRAPH 4e00-9fff
// u4e00
var \u4e00 = 1;
assert.sameValue(一, 1);

// u6c5f, check parser included all CJK range not only first and last
var \u6c5f = 1;
assert.sameValue(江, 1);

// u9fa5, last character in CJK UNIFIED IDEOGRAPH as for 2019
var \u9fa5 = 1;
assert.sameValue(龥, 1);

// CJK UNIFIED IDEOGRAPH EXTENDED A 3400-4dbf
// u3400
var \u3400 = 1;
assert.sameValue(㐀, 1);

// u362e
var \u362e = 1;
assert.sameValue(㘮, 1);

// u4db5, last in CJK UNIFIED IDEOGRAPH EXTENDED A
var \u4db5 = 1;
assert.sameValue(䶵, 1);
