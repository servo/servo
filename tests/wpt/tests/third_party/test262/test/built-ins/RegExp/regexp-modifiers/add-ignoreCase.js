// Copyright 2023 Ron Buckton. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
author: Ron Buckton
description: >
  ignoreCase (`i`) modifier can be added via `(?i:)` or `(?i-:)`.
info: |
  Runtime Semantics: CompileAtom
  The syntax-directed operation CompileAtom takes arguments direction (forward or backward) and modifiers (a Modifiers Record) and returns a Matcher.

  Atom :: `(` `?` RegularExpressionFlags `:` Disjunction `)`
    1. Let addModifiers be the source text matched by RegularExpressionFlags.
    2. Let removeModifiers be the empty String.
    3. Let newModifiers be UpdateModifiers(modifiers, CodePointsToString(addModifiers), removeModifiers).
    4. Return CompileSubpattern of Disjunction with arguments direction and newModifiers.

  UpdateModifiers ( modifiers, add, remove )
  The abstract operation UpdateModifiers takes arguments modifiers (a Modifiers Record), add (a String), and remove (a String) and returns a Modifiers. It performs the following steps when called:

  1. Let dotAll be modifiers.[[DotAll]].
  2. Let ignoreCase be modifiers.[[IgnoreCase]].
  3. Let multiline be modifiers.[[Multiline]].
  4. If add contains "s", set dotAll to true.
  5. If add contains "i", set ignoreCase to true.
  6. If add contains "m", set multiline to true.
  7. If remove contains "s", set dotAll to false.
  8. If remove contains "i", set ignoreCase to false.
  9. If remove contains "m", set multiline to false.
  10. Return the Modifiers Record { [[DotAll]]: dotAll, [[IgnoreCase]]: ignoreCase, [[Multiline]]: multiline }.

esid: sec-compileatom
features: [regexp-modifiers]
---*/

var re1 = /(?i:a)b/;
assert(!re1.test("AB"), "b should not match B in AB");
assert(re1.test("Ab"), "a should match A in AB");
assert(re1.test("ab"), "should match AB");

var re2 = new RegExp("(?i:a)b");
assert(!re2.test("AB"), "b should not match B in AB");
assert(re2.test("Ab"), "a should match A in AB");
assert(re2.test("ab"), "should match AB");

var re3 = /b(?i:a)/;
assert(!re3.test("BA"), "b should not match B in BA");
assert(re3.test("bA"), "a should match A in BA");
assert(re3.test("ba"), "should match BA");

var re4 = new RegExp("b(?i:a)");
assert(!re4.test("BA"), "b should not match B in BA");
assert(re4.test("bA"), "a should match A in BA");
assert(re4.test("ba"), "should match BA");

var re5 = /a(?i:b)c/;
assert(re5.test("abc"), "b should match b in abc");
assert(re5.test("aBc"), "B should match b in abc");
assert(!re5.test("ABc"), "A should not match a in abc");
assert(!re5.test("aBC"), "C should not match c in abc");
assert(!re5.test("ABC"), "should not match abc");

var re6 = new RegExp("a(?i:b)c");
assert(re6.test("abc"), "b should match b in abc");
assert(re6.test("aBc"), "B should match b in abc");
assert(!re6.test("ABc"), "A should not match a in abc");
assert(!re6.test("aBC"), "C should not match c in abc");
assert(!re6.test("ABC"), "should not match abc");
