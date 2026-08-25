// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-literals-string-literals
description: String value for LegacyOctalEscapeSequence
info: |
    EscapeSequence ::
      CharacterEscapeSequence
      LegacyOctalEscapeSequence
      NonOctalDecimalEscapeSequence
      HexEscapeSequence
      UnicodeEscapeSequence

    LegacyOctalEscapeSequence ::
      OctalDigit [lookahead ∉ OctalDigit]
      ZeroToThree OctalDigit [lookahead ∉ OctalDigit]
      FourToSeven OctalDigit
      ZeroToThree OctalDigit OctalDigit

    ZeroToThree :: one of
      0 1 2 3

    FourToSeven :: one of
      4 5 6 7
flags: [noStrict]
---*/

// LegacyOctalEscapeSequence ::
//   OctalDigit [lookahead ∉ OctalDigit]
assert.sameValue('\0', '\x00', '\\0');
assert.sameValue('\1', '\x01', '\\1');
assert.sameValue('\2', '\x02', '\\2');
assert.sameValue('\3', '\x03', '\\3');
assert.sameValue('\4', '\x04', '\\4');
assert.sameValue('\5', '\x05', '\\5');
assert.sameValue('\6', '\x06', '\\6');
assert.sameValue('\7', '\x07', '\\7');

assert.sameValue('\08', '\x008', '\\08');
assert.sameValue('\18', '\x018', '\\18');
assert.sameValue('\28', '\x028', '\\28');
assert.sameValue('\38', '\x038', '\\38');
assert.sameValue('\48', '\x048', '\\48');
assert.sameValue('\58', '\x058', '\\58');
assert.sameValue('\68', '\x068', '\\68');
assert.sameValue('\78', '\x078', '\\78');
assert.sameValue('\08', '\x008', '\\08');

// LegacyOctalEscapeSequence ::
//   ZeroToThree OctalDigit [lookahead ∉ OctalDigit]
assert.sameValue('\00', '\x00', '\\00');
assert.sameValue('\01', '\x01', '\\01');
assert.sameValue('\06', '\x06', '\\06');
assert.sameValue('\07', '\x07', '\\07');
assert.sameValue('\10', '\x08', '\\10');
assert.sameValue('\11', '\x09', '\\11');
assert.sameValue('\16', '\x0e', '\\16');
assert.sameValue('\17', '\x0f', '\\17');
assert.sameValue('\20', '\x10', '\\20');
assert.sameValue('\21', '\x11', '\\21');
assert.sameValue('\26', '\x16', '\\26');
assert.sameValue('\27', '\x17', '\\27');
assert.sameValue('\30', '\x18', '\\30');
assert.sameValue('\31', '\x19', '\\31');
assert.sameValue('\36', '\x1e', '\\36');
assert.sameValue('\37', '\x1f', '\\37');
assert.sameValue('\008', '\x008', '\\008');
assert.sameValue('\018', '\x018', '\\018');
assert.sameValue('\068', '\x068', '\\068');
assert.sameValue('\078', '\x078', '\\078');
assert.sameValue('\108', '\x088', '\\108');
assert.sameValue('\118', '\x098', '\\118');
assert.sameValue('\168', '\x0e8', '\\168');
assert.sameValue('\178', '\x0f8', '\\178');
assert.sameValue('\208', '\x108', '\\208');
assert.sameValue('\218', '\x118', '\\218');
assert.sameValue('\268', '\x168', '\\268');
assert.sameValue('\278', '\x178', '\\278');
assert.sameValue('\308', '\x188', '\\308');
assert.sameValue('\318', '\x198', '\\318');
assert.sameValue('\368', '\x1e8', '\\368');
assert.sameValue('\378', '\x1f8', '\\378');

// LegacyOctalEscapeSequence ::
//   FourToSeven OctalDigit
assert.sameValue('\40', '\x20', '\\40');
assert.sameValue('\41', '\x21', '\\41');
assert.sameValue('\46', '\x26', '\\46');
assert.sameValue('\47', '\x27', '\\47');
assert.sameValue('\50', '\x28', '\\50');
assert.sameValue('\51', '\x29', '\\51');
assert.sameValue('\56', '\x2e', '\\56');
assert.sameValue('\57', '\x2f', '\\57');
assert.sameValue('\60', '\x30', '\\60');
assert.sameValue('\61', '\x31', '\\61');
assert.sameValue('\66', '\x36', '\\66');
assert.sameValue('\67', '\x37', '\\67');
assert.sameValue('\70', '\x38', '\\70');
assert.sameValue('\71', '\x39', '\\71');
assert.sameValue('\76', '\x3e', '\\76');
assert.sameValue('\77', '\x3f', '\\77');
assert.sameValue('\400', '\x200', '\\400');
assert.sameValue('\410', '\x210', '\\410');
assert.sameValue('\460', '\x260', '\\460');
assert.sameValue('\470', '\x270', '\\470');
assert.sameValue('\500', '\x280', '\\500');
assert.sameValue('\510', '\x290', '\\510');
assert.sameValue('\560', '\x2e0', '\\560');
assert.sameValue('\570', '\x2f0', '\\570');
assert.sameValue('\600', '\x300', '\\600');
assert.sameValue('\610', '\x310', '\\610');
assert.sameValue('\660', '\x360', '\\660');
assert.sameValue('\670', '\x370', '\\670');
assert.sameValue('\700', '\x380', '\\700');
assert.sameValue('\710', '\x390', '\\710');
assert.sameValue('\760', '\x3e0', '\\760');
assert.sameValue('\770', '\x3f0', '\\770');

// LegacyOctalEscapeSequence ::
//   ZeroToThree OctalDigit OctalDigit
assert.sameValue('\000', '\x00', '\\000');
assert.sameValue('\001', '\x01', '\\001');
assert.sameValue('\010', '\x08', '\\010');
assert.sameValue('\006', '\x06', '\\006');
assert.sameValue('\060', '\x30', '\\060');
assert.sameValue('\007', '\x07', '\\007');
assert.sameValue('\070', '\x38', '\\070');
assert.sameValue('\077', '\x3f', '\\077');
assert.sameValue('\100', '\x40', '\\100');
assert.sameValue('\101', '\x41', '\\101');
assert.sameValue('\110', '\x48', '\\110');
assert.sameValue('\106', '\x46', '\\106');
assert.sameValue('\160', '\x70', '\\160');
assert.sameValue('\107', '\x47', '\\107');
assert.sameValue('\170', '\x78', '\\170');
assert.sameValue('\177', '\x7f', '\\177');
assert.sameValue('\200', '\x80', '\\200');
assert.sameValue('\201', '\x81', '\\201');
assert.sameValue('\210', '\x88', '\\210');
assert.sameValue('\206', '\x86', '\\206');
assert.sameValue('\260', '\xb0', '\\260');
assert.sameValue('\207', '\x87', '\\207');
assert.sameValue('\270', '\xb8', '\\270');
assert.sameValue('\277', '\xbf', '\\277');
assert.sameValue('\300', '\xc0', '\\300');
assert.sameValue('\301', '\xc1', '\\301');
assert.sameValue('\310', '\xc8', '\\310');
assert.sameValue('\306', '\xc6', '\\306');
assert.sameValue('\360', '\xf0', '\\360');
assert.sameValue('\307', '\xc7', '\\307');
assert.sameValue('\370', '\xf8', '\\370');
assert.sameValue('\377', '\xff', '\\377');
