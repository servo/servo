// Copyright (C) 2015 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: B.1.4 is not applied for Unicode RegExp - Identity escape with basic latin characters
info: |
    The compatibility extensions defined in B.1.4 Regular Expressions Patterns
    are not applied for Unicode RegExps.
    Tested extension: "IdentityEscape[U] :: [~U] SourceCharacter but not c"
es6id: 21.1.2
---*/

function isSyntaxCharacter(c) {
  switch (c) {
    case "^":
    case "$":
    case "\\":
    case ".":
    case "*":
    case "+":
    case "?":
    case "(":
    case ")":
    case "[":
    case "]":
    case "{":
    case "}":
    case "|":
      return true;
    default:
      return false;
  }
}

function isAlphaDigit(c) {
  return ("0" <= c && c <= "9") || ("A" <= c && c <= "Z") || ("a" <= c && c <= "z");
}


// IdentityEscape in AtomEscape.
//
// AtomEscape[U] :: CharacterEscape[?U]
// CharacterEscape[U] :: IdentityEscape[?U]
for (var cu = 0x00; cu <= 0x7f; ++cu) {
  var s = String.fromCharCode(cu);
  if (!isAlphaDigit(s) && !isSyntaxCharacter(s) && s !== "/") {
    assert.throws(SyntaxError, function() {
      RegExp("\\" + s, "u");
    }, "Invalid IdentityEscape in AtomEscape: '\\" + s + "'");
  }
}


// IdentityEscape in ClassEscape.
//
// ClassEscape[U] :: CharacterEscape[?U]
// CharacterEscape[U] :: IdentityEscape[?U]
for (var cu = 0x00; cu <= 0x7f; ++cu) {
  var s = String.fromCharCode(cu);
  if (!isAlphaDigit(s) && !isSyntaxCharacter(s) && s !== "/" && s !== "-") {
    assert.throws(SyntaxError, function() {
      RegExp("[\\" + s + "]", "u");
    }, "Invalid IdentityEscape in ClassEscape: '\\" + s + "'");
  }
}
