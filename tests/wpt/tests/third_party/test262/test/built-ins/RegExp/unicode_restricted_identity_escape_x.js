// Copyright (C) 2015 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: B.1.4 is not applied for Unicode RegExp - Incomplete hexadecimal escape sequences
info: |
    The compatibility extensions defined in B.1.4 Regular Expressions Patterns
    are not applied for Unicode RegExp.
    Tested extension: "IdentityEscape[U] :: [~U] SourceCharacter but not c"
es6id: 21.1.2
---*/

// Incomplete HexEscapeSequence in AtomEscape not parsed as IdentityEscape.
//
// AtomEscape[U] :: CharacterEscape[?U]
// CharacterEscape[U] :: HexEscapeSequence
assert.throws(SyntaxError, function() {
  RegExp("\\x", "u");
}, 'RegExp("\\x", "u"): ');
assert.throws(SyntaxError, function() {
  RegExp("\\x1", "u");
}, 'RegExp("\\x1", "u"): ');


// Incomplete HexEscapeSequence in ClassEscape not parsed as IdentityEscape.
//
// ClassEscape[U] :: CharacterEscape[?U]
// CharacterEscape[U] :: HexEscapeSequence
assert.throws(SyntaxError, function() {
  RegExp("[\\x]", "u");
}, 'RegExp("[\\x]", "u"): ');
assert.throws(SyntaxError, function() {
  RegExp("[\\x1]", "u");
}, 'RegExp("[\\x1]", "u"): ');
