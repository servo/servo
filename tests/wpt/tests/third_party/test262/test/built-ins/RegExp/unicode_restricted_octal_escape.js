// Copyright (C) 2015 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: B.1.4 is not applied for Unicode RegExp - Octal escape sequences
info: |
    The compatibility extensions defined in B.1.4 Regular Expressions Patterns
    are not applied for Unicode RegExp.
    Tested extension: "CharacterEscape[U] :: [~U] LegacyOctalEscapeSequence"
es6id: 21.1.2
---*/

// DecimalEscape without leading 0 in AtomEscape.
//
// AtomEscape[U] :: DecimalEscape
// DecimalEscape :: DecimalIntegerLiteral [lookahead /= DecimalDigit]
assert.throws(SyntaxError, function() {
  RegExp("\\1", "u");
}, 'RegExp("\\1", "u"): ');
assert.throws(SyntaxError, function() {
  RegExp("\\2", "u");
}, 'RegExp("\\2", "u"): ');
assert.throws(SyntaxError, function() {
  RegExp("\\3", "u");
}, 'RegExp("\\3", "u"): ');
assert.throws(SyntaxError, function() {
  RegExp("\\4", "u");
}, 'RegExp("\\4", "u"): ');
assert.throws(SyntaxError, function() {
  RegExp("\\5", "u");
}, 'RegExp("\\5", "u"): ');
assert.throws(SyntaxError, function() {
  RegExp("\\6", "u");
}, 'RegExp("\\6", "u"): ');
assert.throws(SyntaxError, function() {
  RegExp("\\7", "u");
}, 'RegExp("\\7", "u"): ');
assert.throws(SyntaxError, function() {
  RegExp("\\8", "u");
}, 'RegExp("\\8", "u"): ');
assert.throws(SyntaxError, function() {
  RegExp("\\9", "u");
}, 'RegExp("\\9", "u"): ');


// DecimalEscape without leading 0 in ClassEscape.
//
// ClassEscape[U] :: DecimalEscape
// DecimalEscape :: DecimalIntegerLiteral [lookahead /= DecimalDigit]
assert.throws(SyntaxError, function() {
  RegExp("[\\1]", "u");
}, 'RegExp("[\\1]", "u"): ');
assert.throws(SyntaxError, function() {
  RegExp("[\\2]", "u");
}, 'RegExp("[\\2]", "u"): ');
assert.throws(SyntaxError, function() {
  RegExp("[\\3]", "u");
}, 'RegExp("[\\3]", "u"): ');
assert.throws(SyntaxError, function() {
  RegExp("[\\4]", "u");
}, 'RegExp("[\\4]", "u"): ');
assert.throws(SyntaxError, function() {
  RegExp("[\\5]", "u");
}, 'RegExp("[\\5]", "u"): ');
assert.throws(SyntaxError, function() {
  RegExp("[\\6]", "u");
}, 'RegExp("[\\6]", "u"): ');
assert.throws(SyntaxError, function() {
  RegExp("[\\7]", "u");
}, 'RegExp("[\\7]", "u"): ');
assert.throws(SyntaxError, function() {
  RegExp("[\\8]", "u");
}, 'RegExp("[\\8]", "u"): ');
assert.throws(SyntaxError, function() {
  RegExp("[\\9]", "u");
}, 'RegExp("[\\9]", "u"): ');


// DecimalEscape with leading 0 in AtomEscape.
//
// Atom[U] :: DecimalEscape
// DecimalEscape :: DecimalIntegerLiteral [lookahead /= DecimalDigit]
assert.throws(SyntaxError, function() {
  RegExp("\\00", "u");
}, 'RegExp("\\00", "u"): ');
assert.throws(SyntaxError, function() {
  RegExp("\\01", "u");
}, 'RegExp("\\01", "u"): ');
assert.throws(SyntaxError, function() {
  RegExp("\\02", "u");
}, 'RegExp("\\02", "u"): ');
assert.throws(SyntaxError, function() {
  RegExp("\\03", "u");
}, 'RegExp("\\03", "u"): ');
assert.throws(SyntaxError, function() {
  RegExp("\\04", "u");
}, 'RegExp("\\04", "u"): ');
assert.throws(SyntaxError, function() {
  RegExp("\\05", "u");
}, 'RegExp("\\05", "u"): ');
assert.throws(SyntaxError, function() {
  RegExp("\\06", "u");
}, 'RegExp("\\06", "u"): ');
assert.throws(SyntaxError, function() {
  RegExp("\\07", "u");
}, 'RegExp("\\07", "u"): ');
assert.throws(SyntaxError, function() {
  RegExp("\\08", "u");
}, 'RegExp("\\08", "u"): ');
assert.throws(SyntaxError, function() {
  RegExp("\\09", "u");
}, 'RegExp("\\09", "u"): ');


// DecimalEscape with leading 0 in ClassEscape.
//
// ClassEscape[U] :: DecimalEscape
// DecimalEscape :: DecimalIntegerLiteral [lookahead /= DecimalDigit]
assert.throws(SyntaxError, function() {
  RegExp("[\\00]", "u");
}, 'RegExp("[\\00]", "u"): ');
assert.throws(SyntaxError, function() {
  RegExp("[\\01]", "u");
}, 'RegExp("[\\01]", "u"): ');
assert.throws(SyntaxError, function() {
  RegExp("[\\02]", "u");
}, 'RegExp("[\\02]", "u"): ');
assert.throws(SyntaxError, function() {
  RegExp("[\\03]", "u");
}, 'RegExp("[\\03]", "u"): ');
assert.throws(SyntaxError, function() {
  RegExp("[\\04]", "u");
}, 'RegExp("[\\04]", "u"): ');
assert.throws(SyntaxError, function() {
  RegExp("[\\05]", "u");
}, 'RegExp("[\\05]", "u"): ');
assert.throws(SyntaxError, function() {
  RegExp("[\\06]", "u");
}, 'RegExp("[\\06]", "u"): ');
assert.throws(SyntaxError, function() {
  RegExp("[\\07]", "u");
}, 'RegExp("[\\07]", "u"): ');
assert.throws(SyntaxError, function() {
  RegExp("[\\08]", "u");
}, 'RegExp("[\\08]", "u"): ');
assert.throws(SyntaxError, function() {
  RegExp("[\\09]", "u");
}, 'RegExp("[\\09]", "u"): ');
