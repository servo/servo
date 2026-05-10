// Copyright (C) 2015 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: B.1.4 is not applied for Unicode RegExp - ClassEscape in range expression
info: |
    The compatibility extensions defined in B.1.4 Regular Expressions Patterns
    are not applied for Unicode RegExp.
    Tested extension: "ClassAtomNoDashInRange :: \ ClassEscape but only if ClassEscape evaluates to a CharSet with exactly one character"
es6id: 21.2.2.15.1
---*/

// Leading CharacterClassEscape.
assert.throws(SyntaxError, function() {
  RegExp("[\\d-a]", "u");
}, 'RegExp("[\\d-a]", "u"): ');
assert.throws(SyntaxError, function() {
  RegExp("[\\D-a]", "u");
}, 'RegExp("[\\D-a]", "u"): ');
assert.throws(SyntaxError, function() {
  RegExp("[\\s-a]", "u");
}, 'RegExp("[\\s-a]", "u"): ');
assert.throws(SyntaxError, function() {
  RegExp("[\\S-a]", "u");
}, 'RegExp("[\\S-a]", "u"): ');
assert.throws(SyntaxError, function() {
  RegExp("[\\w-a]", "u");
}, 'RegExp("[\\w-a]", "u"): ');
assert.throws(SyntaxError, function() {
  RegExp("[\\W-a]", "u");
}, 'RegExp("[\\W-a]", "u"): ');


// Trailing CharacterClassEscape.
assert.throws(SyntaxError, function() {
  RegExp("[a-\\d]", "u");
}, 'RegExp("[a-\\d]", "u"): ');
assert.throws(SyntaxError, function() {
  RegExp("[a-\\D]", "u");
}, 'RegExp("[a-\\D]", "u"): ');
assert.throws(SyntaxError, function() {
  RegExp("[a-\\s]", "u");
}, 'RegExp("[a-\\s]", "u"): ');
assert.throws(SyntaxError, function() {
  RegExp("[a-\\S]", "u");
}, 'RegExp("[a-\\S]", "u"): ');
assert.throws(SyntaxError, function() {
  RegExp("[a-\\w]", "u");
}, 'RegExp("[a-\\w]", "u"): ');
assert.throws(SyntaxError, function() {
  RegExp("[a-\\W]", "u");
}, 'RegExp("[a-\\W]", "u"): ');


// Leading and trailing CharacterClassEscape.
assert.throws(SyntaxError, function() {
  RegExp("[\\d-\\d]", "u");
}, 'RegExp("[\\d-\\d]", "u"): ');
assert.throws(SyntaxError, function() {
  RegExp("[\\D-\\D]", "u");
}, 'RegExp("[\\D-\\D]", "u"): ');
assert.throws(SyntaxError, function() {
  RegExp("[\\s-\\s]", "u");
}, 'RegExp("[\\s-\\s]", "u"): ');
assert.throws(SyntaxError, function() {
  RegExp("[\\S-\\S]", "u");
}, 'RegExp("[\\S-\\S]", "u"): ');
assert.throws(SyntaxError, function() {
  RegExp("[\\w-\\w]", "u");
}, 'RegExp("[\\w-\\w]", "u"): ');
assert.throws(SyntaxError, function() {
  RegExp("[\\W-\\W]", "u");
}, 'RegExp("[\\W-\\W]", "u"): ');
