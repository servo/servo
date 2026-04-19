// Copyright 2017 the V8 project authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: prod-annexB-ClassAtomNoDash
description: >
  Character classes containing an invalid control escape behave like [\\c]
info: |
  ClassAtomNoDash :: `\`

  The production ClassAtomNoDash :: `\` evaluates as follows:
    1. Return the CharSet containing the single character `\`.
features: [generators]
---*/

function* invalidControls() {
  // Check ASCII characters which are not in the extended range or syntax
  // characters
  for (let alpha = 0x00; alpha <= 0x7F; alpha++) {
    let letter = String.fromCharCode(alpha);
    if (!letter.match(/[0-9A-Za-z_\$(|)\[\]\/\\^]/)) {
      yield letter;
    }
  }
  yield "";
}

for (let letter of invalidControls()) {
  var source = "[\\c" + letter + "]";
  var re = new RegExp(source);

  if (letter.length > 0) {
    var char = letter.charCodeAt(0);
    var str = String.fromCharCode(char % 32);
    var arr = re.exec(str);
    if (str !== letter && arr !== null) {
      throw new Test262Error(`Character ${letter} unreasonably wrapped around as a control character`);
    }

    arr = re.exec(letter);
    if (arr === null) {
      throw new Test262Error(`Character ${letter} missing from character class ${source}`);
    }
  }
  arr = re.exec("\\")
  if (arr === null) {
    throw new Test262Error(`Character \\ missing from character class ${source}`);
  }
  arr = re.exec("c")
  if (arr === null) {
    throw new Test262Error(`Character c missing from character class ${source}`);
  }
}

