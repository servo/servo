// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-literals-numeric-literals
description: Mathematical value for NonOctalDecimalIntegerLiteral
info: |
     DecimalIntegerLiteral ::
       0
       NonZeroDigit DecimalDigits[opt]
       NonOctalDecimalIntegerLiteral

     NonOctalDecimalIntegerLiteral ::
       0 NonOctalDigit
       LegacyOctalLikeDecimalIntegerLiteral NonOctalDigit
       NonOctalDecimalIntegerLiteral DecimalDigit

     LegacyOctalLikeDecimalIntegerLiteral ::
       0 OctalDigit
       LegacyOctalLikeDecimalIntegerLiteral OctalDigit

     NonOctalDigit :: one of
       8 9
flags: [noStrict]
---*/

// NonOctalDecimalIntegerLiteral ::
//   0 NonOctalDigit
assert.sameValue(08, 8, '08');
assert.sameValue(09, 9, '09');

// NonOctalDecimalIntegerLiteral ::
//   LegacyOctalLikeDecimalIntegerLiteral NonOctalDigit
assert.sameValue(008, 8, '008');
assert.sameValue(018, 18, '018');
assert.sameValue(028, 28, '028');
assert.sameValue(038, 38, '038');
assert.sameValue(048, 48, '048');
assert.sameValue(058, 58, '058');
assert.sameValue(068, 68, '068');
assert.sameValue(078, 78, '078');
assert.sameValue(088, 88, '088');
assert.sameValue(098, 98, '098');
assert.sameValue(0708, 708, '708');
assert.sameValue(0718, 718, '718');
assert.sameValue(0728, 728, '728');
assert.sameValue(0738, 738, '738');
assert.sameValue(0748, 748, '748');
assert.sameValue(0758, 758, '758');
assert.sameValue(0768, 768, '768');
assert.sameValue(0778, 778, '778');
assert.sameValue(0788, 788, '788');
assert.sameValue(0798, 798, '798');

assert.sameValue(009, 9, '009');
assert.sameValue(019, 19, '019');
assert.sameValue(029, 29, '029');
assert.sameValue(039, 39, '039');
assert.sameValue(049, 49, '049');
assert.sameValue(059, 59, '059');
assert.sameValue(069, 69, '069');
assert.sameValue(079, 79, '079');
assert.sameValue(089, 89, '089');
assert.sameValue(099, 99, '099');
assert.sameValue(0709, 709, '0709');
assert.sameValue(0719, 719, '0719');
assert.sameValue(0729, 729, '0729');
assert.sameValue(0739, 739, '0739');
assert.sameValue(0749, 749, '0749');
assert.sameValue(0759, 759, '0759');
assert.sameValue(0769, 769, '0769');
assert.sameValue(0779, 779, '0779');
assert.sameValue(0789, 789, '0789');
assert.sameValue(0799, 799, '0799');

// NonOctalDecimalIntegerLiteral ::
//   NonOctalDecimalIntegerLiteral DecimalDigit
assert.sameValue(080, 80, '080');
assert.sameValue(081, 81, '081');
assert.sameValue(082, 82, '082');
assert.sameValue(083, 83, '083');
assert.sameValue(084, 84, '084');
assert.sameValue(085, 85, '085');
assert.sameValue(086, 86, '086');
assert.sameValue(087, 87, '087');
assert.sameValue(088, 88, '088');
assert.sameValue(089, 89, '089');

assert.sameValue(0780, 780, '0780');
assert.sameValue(0781, 781, '0781');
assert.sameValue(0782, 782, '0782');
assert.sameValue(0783, 783, '0783');
assert.sameValue(0784, 784, '0784');
assert.sameValue(0785, 785, '0785');
assert.sameValue(0786, 786, '0786');
assert.sameValue(0787, 787, '0787');
assert.sameValue(0788, 788, '0788');
assert.sameValue(0789, 789, '0789');

assert.sameValue(090, 90, '090');
assert.sameValue(091, 91, '091');
assert.sameValue(092, 92, '092');
assert.sameValue(093, 93, '093');
assert.sameValue(094, 94, '094');
assert.sameValue(095, 95, '095');
assert.sameValue(096, 96, '096');
assert.sameValue(097, 97, '097');
assert.sameValue(098, 98, '098');
assert.sameValue(099, 99, '099');

assert.sameValue(0790, 790, '0790');
assert.sameValue(0791, 791, '0791');
assert.sameValue(0792, 792, '0792');
assert.sameValue(0793, 793, '0793');
assert.sameValue(0794, 794, '0794');
assert.sameValue(0795, 795, '0795');
assert.sameValue(0796, 796, '0796');
assert.sameValue(0797, 797, '0797');
assert.sameValue(0798, 798, '0798');
assert.sameValue(0799, 799, '0799');
