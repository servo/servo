// Copyright 2023 Ron Buckton. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
author: Ron Buckton
description: >
  dotAll (`s`) modifier can be added via `(?s:)` or `(?s-:)`.
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

var re1 = /(?-s:^.$)/s;
assert(re1.test("a"), "Pattern character '.' should match non-line terminators in modified group");
assert(re1.test("3"), "Pattern character '.' should match non-line terminators in modified group");
assert(re1.test("π"), "Pattern character '.' should match non-line terminators in modified group");
assert(re1.test("\u2027"), "Pattern character '.' should match non-line terminators in modified group");
assert(re1.test("\u0085"), "Pattern character '.' should match non-line terminators in modified group");
assert(re1.test("\v"), "Pattern character '.' should match mon-line terminators in modified group");
assert(re1.test("\f"), "Pattern character '.' should match mon-line terminators in modified group");
assert(re1.test("\u180E"), "Pattern character '.' should match non-line terminators in modified group");
assert(!re1.test("\u{10300}"), "Supplementary plane not matched by a single .");
assert(!re1.test("\n"), "Pattern character '.' should not match '\\n' in modified group");
assert(!re1.test("\r"), "Pattern character '.' should not match '\\r' in modified group");
assert(!re1.test("\u2028"), "Pattern character '.' should not match '\\u2028' in modified group");
assert(!re1.test("\u2029"), "Pattern character '.' should not match '\\u2029' in modified group");
assert(re1.test("\uD800"), "Pattern character '.' should match non-line terminators in modified group");
assert(re1.test("\uDFFF"), "Pattern character '.' should match non-line terminators in modified group");

var re2 = new RegExp("(?-s:^.$)", "s");
assert(re2.test("a"), "Pattern character '.' should match non-line terminators in modified group");
assert(re2.test("3"), "Pattern character '.' should match non-line terminators in modified group");
assert(re2.test("π"), "Pattern character '.' should match non-line terminators in modified group");
assert(re2.test("\u2027"), "Pattern character '.' should match non-line terminators in modified group");
assert(re2.test("\u0085"), "Pattern character '.' should match non-line terminators in modified group");
assert(re2.test("\v"), "Pattern character '.' should match mon-line terminators in modified group");
assert(re2.test("\f"), "Pattern character '.' should match mon-line terminators in modified group");
assert(re2.test("\u180E"), "Pattern character '.' should match non-line terminators in modified group");
assert(!re2.test("\u{10300}"), "Supplementary plane not matched by a single .");
assert(!re2.test("\n"), "Pattern character '.' should not match '\\n' in modified group");
assert(!re2.test("\r"), "Pattern character '.' should not match '\\r' in modified group");
assert(!re2.test("\u2028"), "Pattern character '.' should not match '\\u2028' in modified group");
assert(!re2.test("\u2029"), "Pattern character '.' should not match '\\u2029' in modified group");
assert(re2.test("\uD800"), "Pattern character '.' should match non-line terminators in modified group");
assert(re2.test("\uDFFF"), "Pattern character '.' should match non-line terminators in modified group");

var re3 = /a.(?-s:b.b).c/s;
assert(re3.test("a,b,b,c"), "Pattern character '.' should match non-line terminators in modified group");
assert(re3.test("a\nb,b\nc"), "Pattern character '.' should match line terminators outside modified group");
assert(!re3.test("a,b\nb,c"), "Pattern character '.' should not match line terminators in modified group");
assert(!re3.test("a\nb\nb\nc"), "Pattern character '.' should not match line terminators in modified group");

var re4 = new RegExp("a.(?-s:b.b).c", "s");
assert(re4.test("a,b,b,c"), "Pattern character '.' should match non-line terminators in modified group");
assert(re4.test("a\nb,b\nc"), "Pattern character '.' should match line terminators outside modified group");
assert(!re4.test("a,b\nb,c"), "Pattern character '.' should not match line terminators in modified group");
assert(!re4.test("a\nb\nb\nc"), "Pattern character '.' should not match line terminators in modified group");
