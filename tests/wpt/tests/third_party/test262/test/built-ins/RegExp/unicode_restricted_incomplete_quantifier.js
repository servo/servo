// Copyright (C) 2015 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: B.1.4 is not applied for Unicode RegExp - Incomplete quantifiers
info: |
    The compatibility extensions defined in B.1.4 Regular Expressions Patterns
    are not applied for Unicode RegExp.
    Tested extension: "Atom[U] :: PatternCharacter"
es6id: 21.1.2
---*/

// Incomplete quantifier with atom.
assert.throws(SyntaxError, function() {
  RegExp("a{", "u");
}, 'RegExp("a{", "u"): ');
assert.throws(SyntaxError, function() {
  RegExp("a{1", "u");
}, 'RegExp("a{1", "u"): ');
assert.throws(SyntaxError, function() {
  RegExp("a{1,", "u");
}, 'RegExp("a{1,", "u"): ');
assert.throws(SyntaxError, function() {
  RegExp("a{1,2", "u");
}, 'RegExp("a{1,2", "u"): ');


// Incomplete quantifier without atom.
assert.throws(SyntaxError, function() {
  RegExp("{", "u");
}, 'RegExp("{", "u"): ');
assert.throws(SyntaxError, function() {
  RegExp("{1", "u");
}, 'RegExp("{1", "u"): ');
assert.throws(SyntaxError, function() {
  RegExp("{1,", "u");
}, 'RegExp("{1,", "u"): ');
assert.throws(SyntaxError, function() {
  RegExp("{1,2", "u");
}, 'RegExp("{1,2", "u"): ');
