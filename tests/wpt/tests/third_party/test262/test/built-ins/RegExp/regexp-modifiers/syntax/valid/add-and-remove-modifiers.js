// Copyright 2023 Ron Buckton. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
author: Ron Buckton
description: >
  Modifiers syntax `(?im-s:)` (and similar) is legal, as long as they do not overlap.
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

/(?i-s:)/;
/(?i-sm:)/;
/(?i-m:)/;
/(?i-ms:)/;
/(?s-i:)/;
/(?s-im:)/;
/(?s-m:)/;
/(?s-mi:)/;
/(?m-i:)/;
/(?m-is:)/;
/(?m-s:)/;
/(?m-si:)/;
/(?is-m:)/;
/(?im-s:)/;
/(?si-m:)/;
/(?sm-i:)/;
/(?mi-s:)/;
/(?ms-i:)/;
new RegExp("(?i-s:)");
new RegExp("(?i-sm:)");
new RegExp("(?i-m:)");
new RegExp("(?i-ms:)");
new RegExp("(?s-i:)");
new RegExp("(?s-im:)");
new RegExp("(?s-m:)");
new RegExp("(?s-mi:)");
new RegExp("(?m-i:)");
new RegExp("(?m-is:)");
new RegExp("(?m-s:)");
new RegExp("(?m-si:)");
new RegExp("(?is-m:)");
new RegExp("(?im-s:)");
new RegExp("(?si-m:)");
new RegExp("(?sm-i:)");
new RegExp("(?mi-s:)");
new RegExp("(?ms-i:)");
