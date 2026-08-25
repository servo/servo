// Copyright 2024 Daniel Kwan. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
author: Daniel Kwan
description: >
  Nesting ignoreCase (`i`) modifier should not affect alternatives outside.
info: |
  Runtime Semantics: CompileAtom
  The syntax-directed operation CompileAtom takes arguments direction (forward or backward) and modifiers (a Modifiers Record) and returns a Matcher.

  Atom :: `(` `?` RegularExpressionFlags `:` Disjunction `)`
    1. Let addModifiers be the source text matched by RegularExpressionFlags.
    2. Let removeModifiers be the empty String.
    3. Let newModifiers be UpdateModifiers(modifiers, CodePointsToString(addModifiers), removeModifiers).
    4. Return CompileSubpattern of Disjunction with arguments direction and newModifiers.

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

var re1 = /a|(?-i:b|(?i:c)|d|(?-i:e)|f)|g|(?i:h)|k/i;
assert(re1.test("A"), "`a` should match `A`");
assert(!re1.test("B"), "`b` should not match `B`");
assert(re1.test("C"), "`c` should match `C`");
assert(!re1.test("D"), "`d` should not match `D`");
assert(!re1.test("E"), "`e` should not match `E`");
assert(!re1.test("F"), "`f` should not match `F`");
assert(re1.test("G"), "`g` should match `G`");
assert(re1.test("H"), "`h` should match `H`");
assert(re1.test("K"), "`k` should match `K`");

var re2 = /a|(?i:b|(?-i:c)|d|(?i:e)|f)|g|(?-i:h)|k/;
assert(!re2.test("A"), "`a` should not match `A`");
assert(re2.test("B"), "`b` should match `B`");
assert(!re2.test("C"), "`c` should not match `C`");
assert(re2.test("D"), "`d` should match `D`");
assert(re2.test("E"), "`e` should match `E`");
assert(re2.test("F"), "`f` should match `F`");
assert(!re2.test("G"), "`g` should not match `G`");
assert(!re2.test("H"), "`h` should not match `H`");
assert(!re2.test("K"), "`k` should not match `K`");
