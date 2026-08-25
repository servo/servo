// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    If Result.type is break and Result.target is in the current
    label set, return (normal, Result.value, empty)
es5id: 12.11_A1_T1
description: Simple test using switch statement
---*/

function SwitchTest(value){
  var result = 0;
  
  switch(value) {
    case 0:
      result += 2;
    case 1:
      result += 4;
      break;
    case 2:
      result += 8;
    case 3:
      result += 16;
    default:
      result += 32;
      break;
    case 4:
      result += 64;
  }
  
  return result;
}
        
if(!(SwitchTest(0) === 6)){
  throw new Test262Error("#1: SwitchTest(0) === 6. Actual:  SwitchTest(0) ==="+ SwitchTest(0)  );
}

if(!(SwitchTest(1) === 4)){
  throw new Test262Error("#2: SwitchTest(1) === 4. Actual:  SwitchTest(1) ==="+ SwitchTest(1)  );
}

if(!(SwitchTest(2) === 56)){
  throw new Test262Error("#3: SwitchTest(2) === 56. Actual:  SwitchTest(2) ==="+ SwitchTest(2)  );
}

if(!(SwitchTest(3) === 48)){
  throw new Test262Error("#4: SwitchTest(3) === 48. Actual:  SwitchTest(3) ==="+ SwitchTest(3)  );
}

if(!(SwitchTest(4) === 64)){
  throw new Test262Error("#5: SwitchTest(4) === 64. Actual:  SwitchTest(4) ==="+ SwitchTest(4)  );
}

if(!(SwitchTest(true) === 32)){
  throw new Test262Error("#6: SwitchTest(true) === 32. Actual:  SwitchTest(true) ==="+ SwitchTest(true)  );
}

if(!(SwitchTest(false) === 32)){
  throw new Test262Error("#7: SwitchTest(false) === 32. Actual:  SwitchTest(false) ==="+ SwitchTest(false)  );
}

if(!(SwitchTest(null) === 32)){
  throw new Test262Error("#8: SwitchTest(null) === 32. Actual:  SwitchTest(null) ==="+ SwitchTest(null)  );
}

if(!(SwitchTest(void 0) === 32)){
  throw new Test262Error("#9: SwitchTest(void 0) === 32. Actual:  SwitchTest(void 0) ==="+ SwitchTest(void 0)  );
}

if(!(SwitchTest('0') === 32)){
  throw new Test262Error("#10: SwitchTest('0') === 32. Actual:  SwitchTest('0') ==="+ SwitchTest('0')  );
}
