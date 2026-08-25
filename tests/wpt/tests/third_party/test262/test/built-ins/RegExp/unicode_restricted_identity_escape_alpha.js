// Copyright (C) 2015 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: B.1.4 is not applied for Unicode RegExp - Identity escape with basic latin letters
info: |
    The compatibility extensions defined in B.1.4 Regular Expressions Patterns
    are not applied for Unicode RegExps.
    Tested extension: "IdentityEscape[U] :: [~U] SourceCharacter but not c"

    Forbidden extension (16.1):
    The RegExp pattern grammars in 21.2.1 and B.1.4 must not be extended to recognize any of the
    source characters A-Z or a-z as IdentityEscape[U] when the U grammar parameter is present.
es6id: 21.1.2
---*/

function isValidAlphaEscapeInAtom(s) {
  switch (s) {
    // Assertion [U] :: \b
    case "b":
    // Assertion [U] :: \B
    case "B":
    // ControlEscape :: one of f n r t v
    case "f":
    case "n":
    case "r":
    case "t":
    case "v":
    // CharacterClassEscape :: one of d D s S w W
    case "d":
    case "D":
    case "s":
    case "S":
    case "w":
    case "W":
      return true;
    default:
      return false;
  }
}

function isValidAlphaEscapeInClass(s) {
  switch (s) {
    // ClassEscape[U] :: b
    case "b":
    // ControlEscape :: one of f n r t v
    case "f":
    case "n":
    case "r":
    case "t":
    case "v":
    // CharacterClassEscape :: one of d D s S w W
    case "d":
    case "D":
    case "s":
    case "S":
    case "w":
    case "W":
      return true;
    default:
      return false;
  }
}

// IdentityEscape in AtomEscape
for (var cu = 0x41 /* A */; cu <= 0x5a /* Z */; ++cu) {
  var s = String.fromCharCode(cu);
  if (!isValidAlphaEscapeInAtom(s)) {
    assert.throws(SyntaxError, function() {
      RegExp("\\" + s, "u");
    }, "IdentityEscape in AtomEscape: '" + s + "'");
  }
}
for (var cu = 0x61 /* a */; cu <= 0x7a /* z */; ++cu) {
  var s = String.fromCharCode(cu);
  if (!isValidAlphaEscapeInAtom(s)) {
    assert.throws(SyntaxError, function() {
      RegExp("\\" + s, "u");
    }, "IdentityEscape in AtomEscape: '" + s + "'");
  }
}


// IdentityEscape in ClassEscape
for (var cu = 0x41 /* A */; cu <= 0x5a /* Z */; ++cu) {
  var s = String.fromCharCode(cu);
  if (!isValidAlphaEscapeInClass(s)) {
    assert.throws(SyntaxError, function() {
      RegExp("[\\" + s + "]", "u");
    }, "IdentityEscape in ClassEscape: '" + s + "'");
  }
}
for (var cu = 0x61 /* a */; cu <= 0x7a /* z */; ++cu) {
  var s = String.fromCharCode(cu);
  if (!isValidAlphaEscapeInClass(s)) {
    assert.throws(SyntaxError, function() {
      RegExp("[\\" + s + "]", "u");
    }, "IdentityEscape in ClassEscape: '" + s + "'");
  }
}
