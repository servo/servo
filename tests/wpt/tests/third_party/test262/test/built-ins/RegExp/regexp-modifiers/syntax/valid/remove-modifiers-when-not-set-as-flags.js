// Copyright 2023 Ron Buckton. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
author: Ron Buckton
description: >
  Modifiers syntax `(?-ims:)` is legal, even when not set as flags.
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

/(?-i:)/;
/(?-is:)/;
/(?-im:)/;
/(?-s:)/;
/(?-si:)/;
/(?-sm:)/;
/(?-m:)/;
/(?-mi:)/;
/(?-ms:)/;
/(?-ims:)/;
/(?-ism:)/;
/(?-smi:)/;
/(?-sim:)/;
/(?-mis:)/;
/(?-msi:)/;
new RegExp("(?-i:)");
new RegExp("(?-is:)");
new RegExp("(?-im:)");
new RegExp("(?-s:)");
new RegExp("(?-si:)");
new RegExp("(?-sm:)");
new RegExp("(?-m:)");
new RegExp("(?-mi:)");
new RegExp("(?-ms:)");
new RegExp("(?-ims:)");
new RegExp("(?-ism:)");
new RegExp("(?-smi:)");
new RegExp("(?-sim:)");
new RegExp("(?-mis:)");
new RegExp("(?-msi:)");
