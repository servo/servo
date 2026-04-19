// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: Check type of various properties
es5id: 11.2.1_A4_T3
description: Checking properties of the Function object
---*/

//CHECK#1-8
if (typeof Function.prototype  !== "function")  throw new Test262Error('#1: typeof Function.prototype === "function". Actual: ' + (typeof Function.prototype ));
if (typeof Function['prototype']  !== "function")  throw new Test262Error('#2: typeof Function["prototype"] === "function". Actual: ' + (typeof Function["prototype"] ));
if (typeof Function.prototype.toString  !== "function")  throw new Test262Error('#3: typeof Function.prototype.toString === "function". Actual: ' + (typeof Function.prototype.toString ));
if (typeof Function.prototype['toString']  !== "function")  throw new Test262Error('#4: typeof Function.prototype["toString"] === "function". Actual: ' + (typeof Function.prototype["toString"] ));
if (typeof Function.prototype.length !== "number")  throw new Test262Error('#5: typeof Function.prototype.length === "number". Actual: ' + (typeof Function.prototype.length ));
if (typeof Function.prototype['length'] !== "number")  throw new Test262Error('#6: typeof Function.prototype["length"] === "number". Actual: ' + (typeof Function.prototype["length"] ));
if (typeof Function.prototype.valueOf  !== "function")  throw new Test262Error('#7: typeof Function.prototype.valueOf === "function". Actual: ' + (typeof Function.prototype.valueOf ));
if (typeof Function.prototype['valueOf']  !== "function")  throw new Test262Error('#8: typeof Function.prototype["valueOf"] === "function". Actual: ' + (typeof Function.prototype["valueOf"] ));
