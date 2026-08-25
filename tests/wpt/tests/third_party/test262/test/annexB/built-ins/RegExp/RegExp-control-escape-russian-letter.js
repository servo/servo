// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: "CharacterEscape :: c ControlLetter"
es5id: 15.10.2.10_A2.1_T3
es6id: B.1.4
description: >
  "ControlLetter :: RUSSIAN ALPHABET is incorrect"
  Instead, fall back to semantics to match literal "\\c"
features: [generators]
---*/

function* invalidControls() {
  // Check upper case Cyrillic
  for (var alpha = 0x0410; alpha <= 0x042F; alpha++) {
    yield String.fromCharCode(alpha);
  }

  // Check lower case Cyrillic
  for (alpha = 0x0430; alpha <= 0x044F; alpha++) {
    yield String.fromCharCode(alpha);
  }

  // Check ASCII characters which are not in the extended range or syntax
  // characters
  for (alpha = 0x00; alpha <= 0x7F; alpha++) {
    let letter = String.fromCharCode(alpha);
    if (!letter.match(/[0-9A-Za-z_\$(|)\[\]\/\\^]/)) {
      yield letter;
    }
  }

  // Check for end of string
  yield "";
}

for (let letter of invalidControls()) {
  var source = "\\c" + letter;
  var re = new RegExp(source);

  if (letter.length > 0) {
    var char = letter.charCodeAt(0);
    var str = String.fromCharCode(char % 32);
    var arr = re.exec(str);
    assert.sameValue(arr, null, `Character ${letter} unreasonably wrapped around as a control character`);
  }
  arr = re.exec(source.substring(1));
  assert.sameValue(arr, null, `invalid \\c escape matched c rather than \\c when followed by ${letter}`);

  arr = re.exec(source);
  assert.notSameValue(arr, null, `invalid \\c escape failed to match \\c when followed by ${letter}`);
}
