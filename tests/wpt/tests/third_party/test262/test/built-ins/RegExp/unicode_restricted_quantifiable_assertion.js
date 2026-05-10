// Copyright (C) 2015 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: B.1.4 is not applied for Unicode RegExp - Production 'QuantifiableAssertion Quantifier'
info: |
    The compatibility extensions defined in B.1.4 Regular Expressions Patterns
    are not applied for Unicode RegExps.
    Tested extension: "ExtendedTerm :: QuantifiableAssertion Quantifier"
es6id: 21.1.2
---*/

// Positive lookahead with quantifier.
assert.throws(SyntaxError, function() {
  RegExp("(?=.)*", "u");
}, 'RegExp("(?=.)*", "u"): ');
assert.throws(SyntaxError, function() {
  RegExp("(?=.)+", "u");
}, 'RegExp("(?=.)+", "u"): ');
assert.throws(SyntaxError, function() {
  RegExp("(?=.)?", "u");
}, 'RegExp("(?=.)?", "u"): ');
assert.throws(SyntaxError, function() {
  RegExp("(?=.){1}", "u");
}, 'RegExp("(?=.){1}", "u"): ');
assert.throws(SyntaxError, function() {
  RegExp("(?=.){1,}", "u");
}, 'RegExp("(?=.){1,}", "u"): ');
assert.throws(SyntaxError, function() {
  RegExp("(?=.){1,2}", "u");
}, 'RegExp("(?=.){1,2}", "u"): ');


// Positive lookahead with reluctant quantifier.
assert.throws(SyntaxError, function() {
  RegExp("(?=.)*?", "u");
}, 'RegExp("(?=.)*?", "u"): ');
assert.throws(SyntaxError, function() {
  RegExp("(?=.)+?", "u");
}, 'RegExp("(?=.)+?", "u"): ');
assert.throws(SyntaxError, function() {
  RegExp("(?=.)??", "u");
}, 'RegExp("(?=.)??", "u"): ');
assert.throws(SyntaxError, function() {
  RegExp("(?=.){1}?", "u");
}, 'RegExp("(?=.){1}?", "u"): ');
assert.throws(SyntaxError, function() {
  RegExp("(?=.){1,}?", "u");
}, 'RegExp("(?=.){1,}?", "u"): ');
assert.throws(SyntaxError, function() {
  RegExp("(?=.){1,2}?", "u");
}, 'RegExp("(?=.){1,2}?", "u"): ');


// Negative lookahead with quantifier.
assert.throws(SyntaxError, function() {
  RegExp("(?!.)*", "u");
}, 'RegExp("(?!.)*", "u"): ');
assert.throws(SyntaxError, function() {
  RegExp("(?!.)+", "u");
}, 'RegExp("(?!.)+", "u"): ');
assert.throws(SyntaxError, function() {
  RegExp("(?!.)?", "u");
}, 'RegExp("(?!.)?", "u"): ');
assert.throws(SyntaxError, function() {
  RegExp("(?!.){1}", "u");
}, 'RegExp("(?!.){1}", "u"): ');
assert.throws(SyntaxError, function() {
  RegExp("(?!.){1,}", "u");
}, 'RegExp("(?!.){1,}", "u"): ');
assert.throws(SyntaxError, function() {
  RegExp("(?!.){1,2}", "u");
}, 'RegExp("(?!.){1,2}", "u"): ');


// Negative lookahead with reluctant quantifier.
assert.throws(SyntaxError, function() {
  RegExp("(?!.)*?", "u");
}, 'RegExp("(?!.)*?", "u"): ');
assert.throws(SyntaxError, function() {
  RegExp("(?!.)+?", "u");
}, 'RegExp("(?!.)+?", "u"): ');
assert.throws(SyntaxError, function() {
  RegExp("(?!.)??", "u");
}, 'RegExp("(?!.)??", "u"): ');
assert.throws(SyntaxError, function() {
  RegExp("(?!.){1}?", "u");
}, 'RegExp("(?!.){1}?", "u"): ');
assert.throws(SyntaxError, function() {
  RegExp("(?!.){1,}?", "u");
}, 'RegExp("(?!.){1,}?", "u"): ');
assert.throws(SyntaxError, function() {
  RegExp("(?!.){1,2}?", "u");
}, 'RegExp("(?!.){1,2}?", "u"): ');
