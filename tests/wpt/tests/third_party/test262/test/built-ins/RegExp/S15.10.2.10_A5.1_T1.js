// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    CharacterEscape :: IdentityEscapeSequence :: SourceCharacter but not
    IdentifierPart
es5id: 15.10.2.10_A5.1_T1
description: "Tested string is \"~`!@#$%^&*()-+={[}]|\\\\:;'<,>./?\" + '\"'"
---*/

var non_ident = "~`!@#$%^&*()-+={[}]|\\:;'<,>./?" + '"';
for (var k = 0; k < non_ident.length; ++k) {
  var arr = new RegExp("\\" + non_ident[k], "g").exec(non_ident);
  assert.notSameValue(arr, null, "No match for character: " + non_ident[k]);
  assert.sameValue(arr[0], non_ident[k]);
}
