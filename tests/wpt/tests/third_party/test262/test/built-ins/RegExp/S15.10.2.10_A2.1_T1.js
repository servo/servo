// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: "CharacterEscape :: c ControlLetter"
es5id: 15.10.2.10_A2.1_T1
description: "ControlLetter :: A - Z"
---*/

var result = true;
for (var alpha = 0x0041; alpha <= 0x005A; alpha++) {
  var str = String.fromCharCode(alpha % 32);
  var arr = (new RegExp("\\c" + String.fromCharCode(alpha))).exec(str);  
  if ((arr === null) || (arr[0] !== str)) {
    result = false;
  }
}

assert.sameValue(result, true, 'The value of result is expected to be true');
