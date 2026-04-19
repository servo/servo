// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    Since LineTerminator between "continue" and Identifier is not allowed,
    "continue" is evaluated without label
es5id: 12.7_A2
description: >
    Checking by using eval, inserting LineTerminator between continue
    and Identifier
---*/

FOR1 : for(var i=1;i<2;i++){
  FOR1NESTED : for(var j=1;j<2;j++) {
    continue
FOR1;
  } while(0);
}

assert.sameValue(j, 2, '#1: Since LineTerminator(U-000A) between continue and Identifier not allowed continue evaluates without label');

FOR2 : for(var i=1;i<2;i++){
  FOR2NESTED : for(var j=1;j<2;j++) {
    continueFOR2;
  } while(0);
}

assert.sameValue(j, 2, '#2: Since LineTerminator(U-000D) between continue and Identifier not allowed continue evaluates without label');

FOR3 : for(var i=1;i<2;i++){
  FOR3NESTED : for(var j=1;j<2;j++) {
    continue FOR3;
  } while(0);
}
assert.sameValue(j, 2, '#3: Since LineTerminator(U-2028) between continue and Identifier not allowed continue evaluates without label');

FOR4 : for(var i=1;i<2;i++){
  FOR4NESTED : for(var j=1;j<2;j++) {
    continue FOR4;
  } while(0);
}

assert.sameValue(j, 2, '#4: Since LineTerminator(U-2029) between continue and Identifier not allowed continue evaluates without label');
