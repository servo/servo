// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
includes: [compareArray.js]
description: |
  Implement RegExp unicode flag -- disallow extended patterns.
info: bugzilla.mozilla.org/show_bug.cgi?id=1135377
esid: pending
---*/

// IdentityEscape

assert.compareArray(/\^\$\\\.\*\+\?\(\)\[\]\{\}\|/u.exec("^$\\.*+?()[]{}|"),
              ["^$\\.*+?()[]{}|"]);
assert.throws(SyntaxError, () => eval(`/\\A/u`));
assert.throws(SyntaxError, () => eval(`/\\-/u`));
assert.throws(SyntaxError, () => eval(`/\\U{10}/u`));
assert.throws(SyntaxError, () => eval(`/\\U0000/u`));
assert.throws(SyntaxError, () => eval(`/\\uD83D\\U0000/u`));

assert.compareArray(/[\^\$\\\.\*\+\?\(\)\[\]\{\}\|]+/u.exec("^$\\.*+?()[]{}|"),
              ["^$\\.*+?()[]{}|"]);
assert.throws(SyntaxError, () => eval(`/[\\A]/u`));
assert.compareArray(/[A\-Z]+/u.exec("a-zABC"),
              ["-"]);
assert.throws(SyntaxError, () => eval(`/[\\U{10}]/u`));
assert.throws(SyntaxError, () => eval(`/[\\U0000]/u`));
assert.throws(SyntaxError, () => eval(`/[\\uD83D\\U0000]/u`));

// PatternCharacter
assert.throws(SyntaxError, () => eval(`/{}/u`));
assert.throws(SyntaxError, () => eval(`/{/u`));
assert.throws(SyntaxError, () => eval(`/}/u`));
assert.throws(SyntaxError, () => eval(`/]/u`));
assert.throws(SyntaxError, () => eval(`/{0}/u`));
assert.throws(SyntaxError, () => eval(`/{1,}/u`));
assert.throws(SyntaxError, () => eval(`/{1,2}/u`));

// QuantifiableAssertion
assert.compareArray(/.B(?=A)/u.exec("cBaCBA"),
              ["CB"]);
assert.compareArray(/.B(?!A)/u.exec("CBAcBa"),
              ["cB"]);
assert.compareArray(/.B(?:A)/u.exec("cBaCBA"),
              ["CBA"]);
assert.compareArray(/.B(A)/u.exec("cBaCBA"),
              ["CBA", "A"]);

assert.throws(SyntaxError, () => eval(`/.B(?=A)+/u`));
assert.throws(SyntaxError, () => eval(`/.B(?!A)+/u`));
assert.compareArray(/.B(?:A)+/u.exec("cBaCBA"),
              ["CBA"]);
assert.compareArray(/.B(A)+/u.exec("cBaCBA"),
              ["CBA", "A"]);

// ControlLetter
assert.compareArray(/\cA/u.exec("\u0001"),
              ["\u0001"]);
assert.compareArray(/\cZ/u.exec("\u001a"),
              ["\u001a"]);
assert.compareArray(/\ca/u.exec("\u0001"),
              ["\u0001"]);
assert.compareArray(/\cz/u.exec("\u001a"),
              ["\u001a"]);

assert.compareArray(/[\cA]/u.exec("\u0001"),
              ["\u0001"]);
assert.compareArray(/[\cZ]/u.exec("\u001a"),
              ["\u001a"]);
assert.compareArray(/[\ca]/u.exec("\u0001"),
              ["\u0001"]);
assert.compareArray(/[\cz]/u.exec("\u001a"),
              ["\u001a"]);

assert.throws(SyntaxError, () => eval(`/\\c/u`));
assert.throws(SyntaxError, () => eval(`/\\c1/u`));
assert.throws(SyntaxError, () => eval(`/\\c_/u`));

assert.throws(SyntaxError, () => eval(`/[\\c]/u`));
assert.throws(SyntaxError, () => eval(`/[\\c1]/u`));
assert.throws(SyntaxError, () => eval(`/[\\c_]/u`));

// HexEscapeSequence
assert.throws(SyntaxError, () => eval(`/\\x/u`));
assert.throws(SyntaxError, () => eval(`/\\x0/u`));
assert.throws(SyntaxError, () => eval(`/\\x1/u`));
assert.throws(SyntaxError, () => eval(`/\\x1G/u`));

assert.throws(SyntaxError, () => eval(`/[\\x]/u`));
assert.throws(SyntaxError, () => eval(`/[\\x0]/u`));
assert.throws(SyntaxError, () => eval(`/[\\x1]/u`));
assert.throws(SyntaxError, () => eval(`/[\\x1G]/u`));

// LegacyOctalEscapeSequence
assert.throws(SyntaxError, () => eval(`/\\52/u`));
assert.throws(SyntaxError, () => eval(`/\\052/u`));

assert.throws(SyntaxError, () => eval(`/[\\52]/u`));
assert.throws(SyntaxError, () => eval(`/[\\052]/u`));

// DecimalEscape
assert.compareArray(/\0/u.exec("\0"),
              ["\0"]);
assert.compareArray(/[\0]/u.exec("\0"),
              ["\0"]);
assert.compareArray(/\0A/u.exec("\0A"),
              ["\0A"]);
assert.compareArray(/\0G/u.exec("\0G"),
              ["\0G"]);
assert.compareArray(/(A.)\1/u.exec("ABACABAB"),
              ["ABAB", "AB"]);
assert.compareArray(/(A.)(B.)(C.)(D.)(E.)(F.)(G.)(H.)(I.)(J.)(K.)\10/u.exec("A1B2C3D4E5F6G7H8I9JaKbJa"),
              ["A1B2C3D4E5F6G7H8I9JaKbJa", "A1", "B2", "C3", "D4", "E5", "F6", "G7", "H8", "I9", "Ja", "Kb"]);

assert.throws(SyntaxError, () => eval(`/\\00/u`));
assert.throws(SyntaxError, () => eval(`/\\01/u`));
assert.throws(SyntaxError, () => eval(`/\\09/u`));
assert.throws(SyntaxError, () => eval(`/\\1/u`));
assert.throws(SyntaxError, () => eval(`/\\2/u`));
assert.throws(SyntaxError, () => eval(`/\\3/u`));
assert.throws(SyntaxError, () => eval(`/\\4/u`));
assert.throws(SyntaxError, () => eval(`/\\5/u`));
assert.throws(SyntaxError, () => eval(`/\\6/u`));
assert.throws(SyntaxError, () => eval(`/\\7/u`));
assert.throws(SyntaxError, () => eval(`/\\8/u`));
assert.throws(SyntaxError, () => eval(`/\\9/u`));
assert.throws(SyntaxError, () => eval(`/\\10/u`));

assert.throws(SyntaxError, () => eval(`/[\\00]/u`));
assert.throws(SyntaxError, () => eval(`/[\\01]/u`));
assert.throws(SyntaxError, () => eval(`/[\\09]/u`));
assert.throws(SyntaxError, () => eval(`/[\\1]/u`));
assert.throws(SyntaxError, () => eval(`/[\\2]/u`));
assert.throws(SyntaxError, () => eval(`/[\\3]/u`));
assert.throws(SyntaxError, () => eval(`/[\\4]/u`));
assert.throws(SyntaxError, () => eval(`/[\\5]/u`));
assert.throws(SyntaxError, () => eval(`/[\\6]/u`));
assert.throws(SyntaxError, () => eval(`/[\\7]/u`));
assert.throws(SyntaxError, () => eval(`/[\\8]/u`));
assert.throws(SyntaxError, () => eval(`/[\\9]/u`));
assert.throws(SyntaxError, () => eval(`/[\\10]/u`));
