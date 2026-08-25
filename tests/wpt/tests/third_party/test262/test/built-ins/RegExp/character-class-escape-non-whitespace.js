// Copyright 2018 Leonardo Balter. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-characterclassescape
description: Detect non WhiteSpace using \S+
info: |
    The production CharacterClassEscape :: S evaluates by returning
    the set of all characters not included in the set returned by
    CharacterClassEscape :: s
---*/

var j;
var i;
var str;
var res;

var whitespaceChars = [
  0x0009,
  0x000A,
  0x000B,
  0x000C,
  0x000D,
  0x0020,
  0x00A0,
  0x1680,
  0x2000,
  0x2001,
  0x2002,
  0x2003,
  0x2004,
  0x2005,
  0x2006,
  0x2007,
  0x2008,
  0x2009,
  0x200A,
  0x2028,
  0x2029,
  0x202F,
  0x205F,
  0x3000,
];

for (j = 0x0000; j < 0x10000; j++) {
  if (j === 0x180E) { continue; } // Skip 0x180E, current test in a separate file
  if (j === 0xFEFF) { continue; } // Ignore BOM
  str = String.fromCharCode(j);
  res = str.replace(/\S+/g, "test262");
  if (whitespaceChars.indexOf(j) >= 0) {
    assert.sameValue(res, str, "WhiteSpace character, charCode: " + j);
  } else {
    assert.sameValue(res, "test262", "Non WhiteSpace character, charCode: " + j);
  }
}
