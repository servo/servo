// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: Sanity test for "catch(Indetifier) statement"
es5id: 12.14_A4
description: Checking if deleting an exception fails
flags: [noStrict]
---*/

// CHECK#1
try {
  throw "catchme";
  throw new Test262Error('#1.1: throw "catchme" lead to throwing exception');
}
catch (e) {
  if (delete e){
    throw new Test262Error('#1.2: Exception has DontDelete property');
  }
  if (e!=="catchme") {
    throw new Test262Error('#1.3: Exception === "catchme". Actual:  Exception ==='+ e  );
  }
}

// CHECK#2
try {
  throw "catchme";
  throw new Test262Error('#2.1: throw "catchme" lead to throwing exception');
}
catch(e){}
try{
  e;
  throw new Test262Error('#2.2: Deleting catching exception after ending "catch" block');
}
catch(err){}
