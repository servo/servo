// Copyright 2023 Ron Buckton. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
author: Ron Buckton
description: >
  New dotAll (`s`) flag from RegExp constructor does not affect dotAll modifier in group.
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

var re1 = new RegExp(/(?s:^.$)/s, "");
assert(re1.test("a"), "Pattern character '.' still should match non-line terminators in modified group");
assert(re1.test("3"), "Pattern character '.' still should match non-line terminators in modified group");
assert(re1.test("π"), "Pattern character '.' still should match non-line terminators in modified group");
assert(re1.test("\u2027"), "Pattern character '.' still should match non-line terminators in modified group");
assert(re1.test("\u0085"), "Pattern character '.' still should match non-line terminators in modified group");
assert(re1.test("\v"), "Pattern character '.' still should match mon-line terminators in modified group");
assert(re1.test("\f"), "Pattern character '.' still should match mon-line terminators in modified group");
assert(re1.test("\u180E"), "Pattern character '.' still should match non-line terminators in modified group");
assert(!re1.test("\u{10300}"), "Supplementary plane still not matched by a single .");
assert(re1.test("\n"), "Pattern character '.' still should match line terminators in modified group");
assert(re1.test("\r"), "Pattern character '.' still should match line terminators in modified group");
assert(re1.test("\u2028"), "Pattern character '.' still should match line terminators in modified group");
assert(re1.test("\u2029"), "Pattern character '.' still should match line terminators in modified group");
assert(re1.test("\uD800"), "Pattern character '.' still should match non-line terminators in modified group");
assert(re1.test("\uDFFF"), "Pattern character '.' still should match non-line terminators in modified group");

var re2 = new RegExp(/(?-s:^.$)/, "s");
assert(re2.test("a"), "Pattern character '.' still should match non-line terminators in modified group");
assert(re2.test("3"), "Pattern character '.' still should match non-line terminators in modified group");
assert(re2.test("π"), "Pattern character '.' still should match non-line terminators in modified group");
assert(re2.test("\u2027"), "Pattern character '.' still should match non-line terminators in modified group");
assert(re2.test("\u0085"), "Pattern character '.' still should match non-line terminators in modified group");
assert(re2.test("\v"), "Pattern character '.' still should match mon-line terminators in modified group");
assert(re2.test("\f"), "Pattern character '.' still should match mon-line terminators in modified group");
assert(re2.test("\u180E"), "Pattern character '.' still should match non-line terminators in modified group");
assert(!re2.test("\u{10300}"), "Supplementary plane still not matched by a single .");
assert(!re2.test("\n"), "Pattern character '.' still should not match '\\n' in modified group");
assert(!re2.test("\r"), "Pattern character '.' still should not match '\\r' in modified group");
assert(!re2.test("\u2028"), "Pattern character '.' still should not match '\\u2028' in modified group");
assert(!re2.test("\u2029"), "Pattern character '.' still should not match '\\u2029' in modified group");
assert(re2.test("\uD800"), "Pattern character '.' still should match non-line terminators in modified group");
assert(re2.test("\uDFFF"), "Pattern character '.' still should match non-line terminators in modified group");
