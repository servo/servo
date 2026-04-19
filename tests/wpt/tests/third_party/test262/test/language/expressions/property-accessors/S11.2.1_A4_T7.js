// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: Check type of various properties
es5id: 11.2.1_A4_T7
description: Checking properties of the Number object
---*/

//CHECK#1-16
if (typeof Number.MAX_VALUE  !== "number")  throw new Test262Error('#1: typeof Number.MAX_VALUE === "number". Actual: ' + (typeof Number.MAX_VALUE ));
if (typeof Number['MAX_VALUE'] !== "number")  throw new Test262Error('#2: typeof Number["MAX_VALUE"] === "number". Actual: ' + (typeof Number["MAX_VALUE"] ));
if (typeof Number.MIN_VALUE !== "number")  throw new Test262Error('#3: typeof Number.MIN_VALUE === "number". Actual: ' + (typeof Number.MIN_VALUE ));
if (typeof Number['MIN_VALUE'] !== "number")  throw new Test262Error('#4: typeof Number["MIN_VALUE"] === "number". Actual: ' + (typeof Number["MIN_VALUE"] ));
if (typeof Number.NaN !== "number")  throw new Test262Error('#5: typeof Number.NaN === "number". Actual: ' + (typeof Number.NaN ));
if (typeof Number['NaN'] !== "number")  throw new Test262Error('#6: typeof Number["NaN"] === "number". Actual: ' + (typeof Number["NaN"] ));
if (typeof Number.NEGATIVE_INFINITY !== "number")  throw new Test262Error('#7: typeof Number.NEGATIVE_INFINITY === "number". Actual: ' + (typeof Number.NEGATIVE_INFINITY ));
if (typeof Number['NEGATIVE_INFINITY'] !== "number")  throw new Test262Error('#8: typeof Number["NEGATIVE_INFINITY"] === "number". Actual: ' + (typeof Number["NEGATIVE_INFINITY"] ));
if (typeof Number.POSITIVE_INFINITY !== "number")  throw new Test262Error('#9: typeof Number.POSITIVE_INFINITY === "number". Actual: ' + (typeof Number.POSITIVE_INFINITY ));
if (typeof Number['POSITIVE_INFINITY'] !== "number")  throw new Test262Error('#10: typeof Number["POSITIVE_INFINITY"] === "number". Actual: ' + (typeof Number["POSITIVE_INFINITY"] ));
if (typeof Number.prototype.toString  !== "function")  throw new Test262Error('#11: typeof Number.prototype.toString === "function". Actual: ' + (typeof Number.prototype.toString ));
if (typeof Number.prototype['toString']  !== "function")  throw new Test262Error('#12: typeof Number.prototype["toString"] === "function". Actual: ' + (typeof Number.prototype["toString"] ));
if (typeof Number.prototype.constructor !== "function")  throw new Test262Error('#13: typeof Number.prototype.constructor === "function". Actual: ' + (typeof Number.prototype.constructor ));
if (typeof Number.prototype['constructor'] !== "function")  throw new Test262Error('#14: typeof Number.prototype["constructor"] === "function". Actual: ' + (typeof Number.prototype["constructor"] ));
if (typeof Number.prototype.valueOf  !== "function")  throw new Test262Error('#15: typeof Number.prototype.valueOf === "function". Actual: ' + (typeof Number.prototype.valueOf ));
if (typeof Number.prototype['valueOf']  !== "function")  throw new Test262Error('#16: typeof Number.prototype["valueOf"] === "function". Actual: ' + (typeof Number.prototype["valueOf"] ));
