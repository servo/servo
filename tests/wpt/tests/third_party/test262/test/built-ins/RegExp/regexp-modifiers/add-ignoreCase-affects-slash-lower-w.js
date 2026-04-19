// Copyright 2023 Ron Buckton. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
author: Ron Buckton
description: >
  Adding ignoreCase (`i`) modifier affects matching for `\w`.
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

  GetWordCharacters ( modifiers )
  The abstract operation GetWordCharacters takes argument modifiers (a Modifiers Record) and returns a CharSet. It performs the following steps when called:

  1. Let wordCharacters be the mathematical set that is the union of all sixty-three characters in "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789_" (letters, numbers, and U+005F (LOW LINE) in the Unicode Basic Latin block) and all characters c for which c is not in that set but Canonicalize(c, modifiers) is.
  2. Return wordCharacters.

esid: sec-compileatom
features: [regexp-modifiers]
---*/

var re1 = /(?i:\w)/;
assert(re1.test("A"), "\\w should match A");
assert(re1.test("a"), "\\w should match a");
assert(re1.test("z"), "\\w should match z");
assert(re1.test("Z"), "\\w should match Z");

var re2 = /(?i:\w)/u;
assert(re2.test("A"), "\\w should match A");
assert(re2.test("a"), "\\w should match a");
assert(re2.test("z"), "\\w should match z");
assert(re2.test("Z"), "\\w should match Z");
assert(re2.test("\u017f"), "\\w should match \u017f");
assert(re2.test("\u212a"), "\\w should match \u212a");

var re3 = /(?i-:\w)/;
assert(re3.test("A"), "\\w should match A");
assert(re3.test("a"), "\\w should match a");
assert(re3.test("z"), "\\w should match z");
assert(re3.test("Z"), "\\w should match Z");

var re4 = /(?i-:\w)/u;
assert(re4.test("A"), "\\w should match A");
assert(re4.test("a"), "\\w should match a");
assert(re4.test("z"), "\\w should match z");
assert(re4.test("Z"), "\\w should match Z");
assert(re4.test("\u017f"), "\\w should match \u017f");
assert(re4.test("\u212a"), "\\w should match \u212a");
