// Copyright 2024 Daniel Kwan. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
author: Daniel Kwan
description: >
  Removing ignoreCase (`i`) modifier should not affect alternatives outside.
info: |
  Runtime Semantics: CompileAtom
  The syntax-directed operation CompileAtom takes arguments direction (forward or backward) and modifiers (a Modifiers Record) and returns a Matcher.

  Atom :: `(` `?` RegularExpressionFlags `-` RegularExpressionFlags `:` Disjunction `)`
    1. Let addModifiers be the source text matched by the first RegularExpressionFlags.
    2. Let removeModifiers be the source text matched by the second RegularExpressionFlags.
    3. Let newModifiers be UpdateModifiers(modifiers, CodePointsToString(addModifiers), CodePointsToString(removeModifiers)).
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

var re1 = /a|b|(?-i:c)|d|e/i;
assert(re1.test("A"), "Alternative `a` should match `A`");
assert(re1.test("B"), "Alternative `b` should match `B`");
assert(!re1.test("C"), "Alternative `(?-i:c)` should not match `C` in modified group");
assert(re1.test("D"), "Alternative `d` should match `D`");
assert(re1.test("E"), "Alternative `e` should match `E`");

var re2 = /(a)|(?:b)|(?-i:c)|(?:d)|(e)/i;
assert(re2.test("A"), "Alternative `(a)` should match `A`");
assert(re2.test("B"), "Alternative `(?:b)` should match `B`");
assert(!re2.test("C"), "Alternative `(?-i:c)` should not match `C` in modified group");
assert(re2.test("D"), "Alternative `(?:d)` should match `D`");
assert(re2.test("E"), "Alternative `(e)` should match `E`");
