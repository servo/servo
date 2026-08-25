// Copyright (C) 2015 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: B.1.4 is not applied for Unicode RegExp - Invalid control escape sequences
info: |
    The compatibility extensions defined in B.1.4 Regular Expressions Patterns
    are not applied for Unicode RegExp.
    Tested extension: "IdentityEscape[U] :: [~U] SourceCharacter but not c"
es6id: 21.1.2
---*/

function isAlpha(c) {
  return ("A" <= c && c <= "Z") || ("a" <= c && c <= "z");
}


// "c ControlLetter" sequence in AtomEscape.
//
// AtomEscape[U] :: CharacterEscape[?U]
// CharacterEscape[U] :: c ControlLetter
assert.throws(SyntaxError, function() { RegExp("\\c", "u"); });
for (var cu = 0x00; cu <= 0x7f; ++cu) {
  var s = String.fromCharCode(cu);
  if (!isAlpha(s)) {
    assert.throws(SyntaxError, function() {
      RegExp("\\c" + s, "u");
    }, "ControlLetter escape in AtomEscape: '" + s + "'");
  }
}


// "c ControlLetter" sequence in ClassEscape.
//
// ClassEscape[U] :: CharacterEscape[?U]
// CharacterEscape[U] :: c ControlLetter
assert.throws(SyntaxError, function() { RegExp("[\\c]", "u"); });
for (var cu = 0x00; cu <= 0x7f; ++cu) {
  var s = String.fromCharCode(cu);
  if (!isAlpha(s)) {
    assert.throws(SyntaxError, function() {
      RegExp("[\\c" + s + "]", "u");
    }, "ControlLetter escape in ClassEscape: '" + s + "'");
  }
}
