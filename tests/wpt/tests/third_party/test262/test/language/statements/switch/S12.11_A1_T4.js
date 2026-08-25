// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    If Result.type is break and Result.target is in the current
    label set, return (normal, Result.value, empty)
es5id: 12.11_A1_T4
description: Using case with isNaN and isNaN(value)
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
    case isNaN(value):
      result += 16;
    default:
      result += 32;
      break;
    case null:
      result += 64;
    case isNaN:
      result += 128;
      break;
    case Infinity:
      result += 256;
    case 2+3:
      result += 512;
      break;
    case undefined:
      result += 1024;
  }
  
  return result;
}

var n = Number(false);

if(!(SwitchTest(n) === 6)){
  throw new Test262Error("#1: SwitchTest(Number(false)) === 6. Actual:  SwitchTest(Number(false)) ==="+ SwitchTest(n)  );
}

if(!(SwitchTest(parseInt) === 32)){
  throw new Test262Error("#2: SwitchTest(parseInt) === 32. Actual:  SwitchTest(parseInt) ==="+ SwitchTest(parseInt)  );
}

if(!(SwitchTest(isNaN) === 128)){
  throw new Test262Error("#3: SwitchTest(isNaN) === 128. Actual:  SwitchTest(isNaN) ==="+ SwitchTest(isNaN)  );
}

if(!(SwitchTest(true) === 32)){
  throw new Test262Error("#6: SwitchTest(true) === 32. Actual:  SwitchTest(true) ==="+ SwitchTest(true)  );
}

if(!(SwitchTest(false) === 48)){
  throw new Test262Error("#7: SwitchTest(false) === 48. Actual:  SwitchTest(false) ==="+ SwitchTest(false)  );
}

if(!(SwitchTest(null) === 192)){
  throw new Test262Error("#8: SwitchTest(null) === 192. Actual:  SwitchTest(null) ==="+ SwitchTest(null)  );
}

if(!(SwitchTest(void 0) === 1024)){
  throw new Test262Error("#9: SwitchTest(void 0) === 1024. Actual:  SwitchTest(void 0) ==="+ SwitchTest(void 0)  );
}

if(!(SwitchTest(NaN) === 32)){
  throw new Test262Error("#10: SwitchTest(NaN) === 32. Actual:  SwitchTest(NaN) ==="+ SwitchTest(NaN)  );
}

if(!(SwitchTest(Infinity) === 768)){
  throw new Test262Error("#10: SwitchTest(NaN) === 768. Actual:  SwitchTest(NaN) ==="+ SwitchTest(NaN)  );
}
