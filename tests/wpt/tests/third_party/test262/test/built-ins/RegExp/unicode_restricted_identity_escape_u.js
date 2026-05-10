// Copyright (C) 2015 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: B.1.4 is not applied for Unicode RegExp - Incomplete Unicode escape sequences
info: |
    The compatibility extensions defined in B.1.4 Regular Expressions Patterns
    are not applied for Unicode RegExp.
    Tested extension: "IdentityEscape[U] :: [~U] SourceCharacter but not c"
es6id: 21.1.2
---*/

// Incomplete RegExpUnicodeEscapeSequence in AtomEscape not parsed as IdentityEscape.
//
// AtomEscape[U] :: CharacterEscape[?U]
// CharacterEscape[U] :: RegExpUnicodeEscapeSequence[?U]
assert.throws(SyntaxError, function() {
  RegExp("\\u", "u");
}, 'RegExp("\\u", "u"): ');
assert.throws(SyntaxError, function() {
  RegExp("\\u1", "u");
}, 'RegExp("\\u1", "u"): ');
assert.throws(SyntaxError, function() {
  RegExp("\\u12", "u");
}, 'RegExp("\\u12", "u"): ');
assert.throws(SyntaxError, function() {
  RegExp("\\u123", "u");
}, 'RegExp("\\u123", "u"): ');
assert.throws(SyntaxError, function() {
  RegExp("\\u{", "u");
}, 'RegExp("\\u{", "u"): ');
assert.throws(SyntaxError, function() {
  RegExp("\\u{}", "u");
}, 'RegExp("\\u{}", "u"): ');
assert.throws(SyntaxError, function() {
  RegExp("\\u{1", "u");
}, 'RegExp("\\u{1", "u"): ');
assert.throws(SyntaxError, function() {
  RegExp("\\u{12", "u");
}, 'RegExp("\\u{12", "u"): ');
assert.throws(SyntaxError, function() {
  RegExp("\\u{123", "u");
}, 'RegExp("\\u{123", "u"): ');


// Incomplete RegExpUnicodeEscapeSequence in ClassEscape not parsed as IdentityEscape.
//
// ClassEscape[U] :: CharacterEscape[?U]
// CharacterEscape[U] :: RegExpUnicodeEscapeSequence[?U]
assert.throws(SyntaxError, function() {
  RegExp("[\\u]", "u");
}, 'RegExp("[\\u]", "u"): ');
assert.throws(SyntaxError, function() {
  RegExp("[\\u1]", "u");
}, 'RegExp("[\\u1]", "u"): ');
assert.throws(SyntaxError, function() {
  RegExp("[\\u12]", "u");
}, 'RegExp("[\\u12]", "u"): ');
assert.throws(SyntaxError, function() {
  RegExp("[\\u123]", "u");
}, 'RegExp("[\\u123]", "u"): ');
assert.throws(SyntaxError, function() {
  RegExp("[\\u{]", "u");
}, 'RegExp("[\\u{]", "u"): ');
assert.throws(SyntaxError, function() {
  RegExp("[\\u{}]", "u");
}, 'RegExp("[\\u{}]", "u"): ');
assert.throws(SyntaxError, function() {
  RegExp("[\\u{1]", "u");
}, 'RegExp("[\\u{1]", "u"): ');
assert.throws(SyntaxError, function() {
  RegExp("[\\u{12]", "u");
}, 'RegExp("[\\u{12]", "u"): ');
assert.throws(SyntaxError, function() {
  RegExp("[\\u{123]", "u");
}, 'RegExp("[\\u{123]", "u"): ');
