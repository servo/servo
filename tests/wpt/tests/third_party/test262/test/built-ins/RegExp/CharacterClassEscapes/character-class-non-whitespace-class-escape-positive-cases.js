// Copyright (C) 2018 Leo Balter.  All rights reserved.
// Copyright (C) 2024 Aurèle Barrière.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: prod-CharacterClassEscape
description: >
  Check positive cases of non-whitespace class escape \S.
info: |
  This is a generated test. Please check out
  https://github.com/tc39/test262/tree/main/tools/regexp-generator/
  for any changes.

  CharacterClassEscape[UnicodeMode] ::
    d
    D
    s
    S
    w
    W
    [+UnicodeMode] p{ UnicodePropertyValueExpression }
    [+UnicodeMode] P{ UnicodePropertyValueExpression }

  22.2.2.9 Runtime Semantics: CompileToCharSet

  CharacterClassEscape :: d
    1. Return the ten-element CharSet containing the characters 0, 1, 2, 3, 4,
      5, 6, 7, 8, and 9.
  CharacterClassEscape :: D
    1. Let S be the CharSet returned by CharacterClassEscape :: d.
    2. Return CharacterComplement(rer, S).
  CharacterClassEscape :: s
    1. Return the CharSet containing all characters corresponding to a code
      point on the right-hand side of the WhiteSpace or LineTerminator
      productions.
  CharacterClassEscape :: S
    1. Let S be the CharSet returned by CharacterClassEscape :: s.
    2. Return CharacterComplement(rer, S).
  CharacterClassEscape :: w
    1. Return MaybeSimpleCaseFolding(rer, WordCharacters(rer)).
  CharacterClassEscape :: W
    1. Let S be the CharSet returned by CharacterClassEscape :: w.
    2. Return CharacterComplement(rer, S).
features: [String.fromCodePoint]
includes: [regExpUtils.js]
flags: [generated]
---*/

const str = buildString(
{
  loneCodePoints: [],
  ranges: [
    [0x00DC00, 0x00DFFF],
    [0x000000, 0x000008],
    [0x00000E, 0x00001F],
    [0x000021, 0x00009F],
    [0x0000A1, 0x00167F],
    [0x001681, 0x001FFF],
    [0x00200B, 0x002027],
    [0x00202A, 0x00202E],
    [0x002030, 0x00205E],
    [0x002060, 0x002FFF],
    [0x003001, 0x00DBFF],
    [0x00E000, 0x00FEFE],
    [0x00FF00, 0x10FFFF]
  ]
}
);

const standard = /^\S+$/;
const unicode = /^\S+$/u;
const vflag = /^\S+$/v;
const regexes = [standard,unicode,vflag];

const errors = [];

for (const regex of regexes) {
  if (!regex.test(str)) {
    // Error, let's find out where
    for (const char of str) {
      if (!regex.test(char)) {
        errors.push('0x' + char.codePointAt(0).toString(16));
      }
    }
  }
}

assert.sameValue(
  errors.length,
  0,
  'Expected full match, but did not match: ' + errors.join(',')
);
