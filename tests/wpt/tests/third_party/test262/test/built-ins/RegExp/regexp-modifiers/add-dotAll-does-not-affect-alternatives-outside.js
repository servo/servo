// Copyright 2024 Daniel Kwan. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
author: Daniel Kwan
description: >
  Adding dotAll (`s`) modifier should not affect alternatives outside.
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

var re1 = /a.a|b.b|(?s:c.c)|d.d|e.e/;
assert(!re1.test("a\na"), "Alternative `a.a` should not match newline");
assert(!re1.test("b\nb"), "Alternative `b.b` should not match newline");
assert(re1.test("c\nc"), "Alternative `(?s:c.c)` should match newline in modified group");
assert(!re1.test("d\nd"), "Alternative `d.d` should not match newline");
assert(!re1.test("e\ne"), "Alternative `e.e` should not match newline");

var re2 = /(a.a)|(?:b.b)|(?s:c.c)|(?:d.d)|(e.e)/;
assert(!re2.test("a\na"), "Alternative `(a.a)` should not match newline");
assert(!re2.test("b\nb"), "Alternative `(?:b.b)` should not match newline");
assert(re2.test("c\nc"), "Alternative `(?s:c.c)` should match newline in modified group");
assert(!re2.test("d\nd"), "Alternative `(?:d.d)` should not match newline");
assert(!re2.test("e\ne"), "Alternative `(e.e)` should not match newline");
