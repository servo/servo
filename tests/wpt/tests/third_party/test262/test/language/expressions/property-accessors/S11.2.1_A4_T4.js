// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: Check type of various properties
es5id: 11.2.1_A4_T4
description: Checking properties of the Array object
---*/

//CHECK#1-8
if (typeof Array.prototype  !== "object")  throw new Test262Error('#1: typeof Array.prototype === "object". Actual: ' + (typeof Array.prototype ));
if (typeof Array['prototype'] !== "object")  throw new Test262Error('#2: typeof Array["prototype"] === "object". Actual: ' + (typeof Array["prototype"] ));
if (typeof Array.length  !== "number")  throw new Test262Error('#3: typeof Array.length === "number". Actual: ' + (typeof Array.length ));
if (typeof Array['length'] !== "number")  throw new Test262Error('#4: typeof Array["length"] === "number". Actual: ' + (typeof Array["length"] ));
if (typeof Array.prototype.constructor  !== "function")  throw new Test262Error('#5: typeof Array.prototype.constructor === "function". Actual: ' + (typeof Array.prototype.constructor ));
if (typeof Array.prototype['constructor'] !== "function")  throw new Test262Error('#6: typeof Array.prototype["constructor"] === "function". Actual: ' + (typeof Array.prototype["constructor"] ));
if (typeof Array.prototype.toString  !== "function")  throw new Test262Error('#7: typeof Array.prototype.toString === "function". Actual: ' + (typeof Array.prototype.toString ));
if (typeof Array.prototype['toString'] !== "function")  throw new Test262Error('#8: typeof Array.prototype["toString"] === "function". Actual: ' + (typeof Array.prototype["toString"] ));
if (typeof Array.prototype.join  !== "function")  throw new Test262Error('#9: typeof Array.prototype.join === "function". Actual: ' + (typeof Array.prototype.join ));
if (typeof Array.prototype['join'] !== "function")  throw new Test262Error('#10: typeof Array.prototype["join"] === "function". Actual: ' + (typeof Array.prototype["join"] ));
if (typeof Array.prototype.reverse  !== "function")  throw new Test262Error('#11: typeof Array.prototype.reverse === "function". Actual: ' + (typeof Array.prototype.reverse ));
if (typeof Array.prototype['reverse'] !== "function")  throw new Test262Error('#12: typeof Array.prototype["reverse"] === "function". Actual: ' + (typeof Array.prototype["reverse"] ));
if (typeof Array.prototype.sort  !== "function")  throw new Test262Error('#13: typeof Array.prototype.sort === "function". Actual: ' + (typeof Array.prototype.sort ));
if (typeof Array.prototype['sort'] !== "function")  throw new Test262Error('#14: typeof Array.prototype["sort"] === "function". Actual: ' + (typeof Array.prototype["sort"] ));
