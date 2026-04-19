// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: Catching objects with try/catch/finally statement
es5id: 12.14_A18_T3
description: Catching boolean
---*/

// CHECK#1
try{
  throw true;
}
catch(e){
  if (e!==true) throw new Test262Error('#1: Exception ===true. Actual:  Exception ==='+ e  );
}

// CHECK#2
try{
  throw false;
}
catch(e){
  if (e!==false) throw new Test262Error('#2: Exception ===false. Actual:  Exception ==='+ e  );
}

// CHECK#3
var b=false;
try{
  throw b;
}
catch(e){
  if (e!==false) throw new Test262Error('#3: Exception ===false. Actual:  Exception ==='+ e  );
}

// CHECK#4
var b=true;
try{
  throw b;
}
catch(e){
  if (e!==true) throw new Test262Error('#4: Exception ===true. Actual:  Exception ==='+ e  );
}

// CHECK#5
var b=true;
try{
  throw b&&false;
}
catch(e){
  if (e!==false) throw new Test262Error('#5: Exception ===false. Actual:  Exception ==='+ e  );
}

// CHECK#5
var b=true;
try{
  throw b||false;
}
catch(e){
  if (e!==true) throw new Test262Error('#6: Exception ===true. Actual:  Exception ==='+ e  );
}
