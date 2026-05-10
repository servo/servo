// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    Since LineTerminator between "break" and Identifier is not allowed,
    "break" is evaluated without label
es5id: 12.8_A2
description: >
    Checking by using eval, inserting LineTerminator between break and
    Identifier
---*/

FOR1 : for(var i=1;i<2;i++){
  LABEL1 : do {
    break
FOR1;
  } while(0);
}

assert.sameValue(i, 2, '#1: Since LineTerminator(U-000A) between break and Identifier not allowed break evaluates without label');

FOR2 : for(var i=1;i<2;i++){
  LABEL2 : do {
    breakFOR2;
  } while(0);
}

assert.sameValue(i, 2, '#2: Since LineTerminator(U-000D) between break and Identifier not allowed break evaluates without label');

FOR3 : for(var i=1;i<2;i++){
  LABEL3 : do {
    break FOR3;
  } while(0);
}

assert.sameValue(i, 2, '#3: Since LineTerminator(U-2028) between break and Identifier not allowed break evaluates without label');

FOR4 : for(var i=1;i<2;i++){
  LABEL4 : do {
    break FOR4;
  } while(0);
}

assert.sameValue(i, 2, '#4: Since LineTerminator(U-2029) between break and Identifier not allowed break evaluates without label');
