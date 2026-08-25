// Copyright 2023 Ron Buckton. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
author: Ron Buckton
description: >
  multiline (`m`) modifier can be added via `(?m:)` or `(?m-:)`.
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

var re1 = /(?m:es$)/;
assert(re1.test("es\ns"), "$ should match newline in modified group");

var re2 = new RegExp("(?m:es$)");
assert(re2.test("es\ns"), "$ should match newline in modified group");

var re3 = /(?m-:es$)/;
assert(re3.test("es\ns"), "$ should match newline in modified group");

var re4 = new RegExp("(?m-:es$)");
assert(re4.test("es\ns"), "$ should match newline in modified group");

var re5 = /^a\n(?m:^b$)\nc$/;
assert(re5.test("a\nb\nc"), "^ and $ should match newline in modified group");
assert(!re5.test("\na\nb\nc"), "^ should not match newline outside modified group");
assert(!re5.test("a\nb\nc\n"), "$ should not match newline outside modified group");
assert(!re5.test("\na\nb\nc\n"), "^ and $ should not match newline outside modified group");

var re6 = new RegExp("^a\\n(?m:^b$)\\nc$");
assert(re6.test("a\nb\nc"), "^ and $ should match newline in modified group");
assert(!re6.test("\na\nb\nc"), "^ should not match newline outside modified group");
assert(!re6.test("a\nb\nc\n"), "$ should not match newline outside modified group");
assert(!re6.test("\na\nb\nc\n"), "^ and $ should not match newline outside modified group");
