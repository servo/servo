// Copyright (C) 2015 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: B.1.4 is not applied for Unicode RegExp - Standalone brackets
info: |
    The compatibility extensions defined in B.1.4 Regular Expressions Patterns
    are not applied for Unicode RegExp.
    Tested extension: "Atom[U] :: PatternCharacter"
es6id: 21.1.2
---*/

// Single parentheses and brackets.
assert.throws(SyntaxError, function() {
  RegExp("(", "u");
}, 'RegExp("(", "u"): ');
assert.throws(SyntaxError, function() {
  RegExp(")", "u");
}, 'RegExp(")", "u"): ');
assert.throws(SyntaxError, function() {
  RegExp("[", "u");
}, 'RegExp("[", "u"): ');
assert.throws(SyntaxError, function() {
  RegExp("]", "u");
}, 'RegExp("]", "u"): ');
assert.throws(SyntaxError, function() {
  RegExp("{", "u");
}, 'RegExp("{", "u"): ');
assert.throws(SyntaxError, function() {
  RegExp("}", "u");
}, 'RegExp("}", "u"): ');
