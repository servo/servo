// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: Operator "in" calls ToString(ShiftExpression)
es5id: 11.8.7_A4
description: Checking ToString coversion;
---*/

//CHECK#1
var object = {};
object["true"] = 1;
if (true in object !== "true" in object) {  
  throw new Test262Error('#1: "var object = {}; object["true"] = 1; true in object === "true" in object');  
}

//CHECK#2
var object = {};
object.Infinity = 1;
if (Infinity in object !== "Infinity" in object) {  
  throw new Test262Error('#2: "var object = {}; object.Infinity = 1; Infinity in object === "Infinity" in object');  
}

//CHECK#4
var object = {};
object.undefined = 1;
if (undefined in object !== "undefined" in object) {  
  throw new Test262Error('#4: "var object = {}; object.undefined = 1; undefined in object === "undefined" in object');  
}

//CHECK#5
var object = {};
object["null"] = 1;
if (null in object !== "null" in object) {  
  throw new Test262Error('#5: "var object = {}; object["null"] = 1; null in object === "null" in object');  
}
