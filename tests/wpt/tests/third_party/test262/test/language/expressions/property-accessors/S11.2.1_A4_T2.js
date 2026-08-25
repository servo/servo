// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: Check type of various properties
es5id: 11.2.1_A4_T2
description: Checking properties and methods of Object objects
---*/

//CHECK#1-8
if (typeof Object.prototype  !== "object")  throw new Test262Error('#1: typeof Object.prototype === "object". Actual: ' + (typeof Object.prototype ));
if (typeof Object['prototype'] !== "object")  throw new Test262Error('#2: typeof Object["prototype"] === "object". Actual: ' + (typeof Object["prototype"] ));
if (typeof Object.toString !== "function")  throw new Test262Error('#3: typeof Object.toString === "function". Actual: ' + (typeof Object.toString ));
if (typeof Object['toString'] !== "function")  throw new Test262Error('#4: typeof Object["toString"] === "function". Actual: ' + (typeof Object["toString"] ));
if (typeof Object.valueOf !== "function")  throw new Test262Error('#5: typeof Object.valueOf === "function". Actual: ' + (typeof Object.valueOf ));
if (typeof Object['valueOf'] !== "function")  throw new Test262Error('#6: typeof Object["valueOf"] === "function". Actual: ' + (typeof Object["valueOf"] ));
if (typeof Object.constructor  !== "function")  throw new Test262Error('#7: typeof Object.constructor === "function". Actual: ' + (typeof Object.constructor ));
if (typeof Object['constructor'] !== "function")  throw new Test262Error('#8: typeof Object["constructor"] === "function". Actual: ' + (typeof Object["constructor"] ));
