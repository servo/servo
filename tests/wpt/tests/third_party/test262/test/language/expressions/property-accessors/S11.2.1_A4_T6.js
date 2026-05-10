// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: Check type of various properties
es5id: 11.2.1_A4_T6
description: Checking properties of the Boolean object
---*/

//CHECK#1-8
if (typeof Boolean.prototype  !== "object")   throw new Test262Error('#1: typeof Boolean.prototype === "object". Actual: ' + (typeof Boolean.prototype ));
if (typeof Boolean['prototype']  !== "object")  throw new Test262Error('#2: typeof Boolean["prototype"] === "object". Actual: ' + (typeof Boolean["prototype"] ));
if (typeof Boolean.constructor  !== "function")  throw new Test262Error('#3: typeof Boolean.constructor === "function". Actual: ' + (typeof Boolean.constructor ));
if (typeof Boolean['constructor']  !== "function")  throw new Test262Error('#4: typeof Boolean["constructor"] === "function". Actual: ' + (typeof Boolean["constructor"] ));
if (typeof Boolean.prototype.valueOf  !== "function")  throw new Test262Error('#5: typeof Boolean.prototype.valueOf === "function". Actual: ' + (typeof Boolean.prototype.valueOf ));
if (typeof Boolean.prototype['valueOf'] !== "function")  throw new Test262Error('#6: typeof Boolean.prototype["valueOf"] === "function". Actual: ' + (typeof Boolean.prototype["valueOf"] ));
if (typeof Boolean.prototype.toString !== "function")  throw new Test262Error('#7: typeof Boolean.prototype.toString === "function". Actual: ' + (typeof Boolean.prototype.toString ));
if (typeof Boolean.prototype['toString'] !== "function")  throw new Test262Error('#8: typeof Boolean.prototype["toString"] === "function". Actual: ' + (typeof Boolean.prototype["toString"] ));
