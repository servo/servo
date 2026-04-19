// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: Check type of various properties
es5id: 11.2.1_A4_T5
description: Checking properties of the String object
---*/

//CHECK#1-28
if (typeof String.prototype  !== "object")  throw new Test262Error('#1: typeof String.prototype === "object". Actual: ' + (typeof String.prototype ));
if (typeof String['prototype']  !== "object")  throw new Test262Error('#2: typeof String["prototype"] === "object". Actual: ' + (typeof String["prototype"] ));
if (typeof String.fromCharCode  !== "function")  throw new Test262Error('#3: typeof String.fromCharCode === "function". Actual: ' + (typeof String.fromCharCode ));
if (typeof String['fromCharCode']  !== "function")  throw new Test262Error('#4: typeof String["fromCharCode"] === "function". Actual: ' + (typeof String["fromCharCode"] ));
if (typeof String.prototype.toString  !== "function")  throw new Test262Error('#5: typeof String.prototype.toString === "function". Actual: ' + (typeof String.prototype.toString ));
if (typeof String.prototype['toString']  !== "function")  throw new Test262Error('#6: typeof String.prototype["toString"] === "function". Actual: ' + (typeof String.prototype["toString"] ));
if (typeof String.prototype.constructor  !== "function")  throw new Test262Error('#7: typeof String.prototype.constructor === "function". Actual: ' + (typeof String.prototype.constructor ));
if (typeof String.prototype['constructor']  !== "function")  throw new Test262Error('#8: typeof String.prototype["constructor"] === "function". Actual: ' + (typeof String.prototype["constructor"] ));
if (typeof String.prototype.valueOf  !== "function")  throw new Test262Error('#9: typeof String.prototype.valueOf === "function". Actual: ' + (typeof String.prototype.valueOf ));
if (typeof String.prototype['valueOf']  !== "function")  throw new Test262Error('#10: typeof String.prototype["valueOf"] === "function". Actual: ' + (typeof String.prototype["valueOf"] ));
if (typeof String.prototype.charAt !== "function")  throw new Test262Error('#11: typeof String.prototype.charAt === "function". Actual: ' + (typeof String.prototype.charAt ));
if (typeof String.prototype['charAt'] !== "function")  throw new Test262Error('#12: typeof String.prototype["charAt"] === "function". Actual: ' + (typeof String.prototype["charAt"] ));
if (typeof String.prototype.charCodeAt !== "function")  throw new Test262Error('#13: typeof String.prototype.charCodeAt === "function". Actual: ' + (typeof String.prototype.charCodeAt ));
if (typeof String.prototype['charCodeAt'] !== "function")  throw new Test262Error('#14: typeof String.prototype["charCodeAt"] === "function". Actual: ' + (typeof String.prototype["charCodeAt"] ));
if (typeof String.prototype.indexOf  !== "function")  throw new Test262Error('#15: typeof String.prototype.indexOf === "function". Actual: ' + (typeof String.prototype.indexOf ));
if (typeof String.prototype['indexOf']  !== "function")  throw new Test262Error('#16: typeof String.prototype["indexOf"] === "function". Actual: ' + (typeof String.prototype["indexOf"] ));
if (typeof String.prototype.lastIndexOf  !== "function")  throw new Test262Error('#17: typeof String.prototype.lastIndexOf === "function". Actual: ' + (typeof String.prototype.lastIndexOf ));
if (typeof String.prototype['lastIndexOf']  !== "function")  throw new Test262Error('#18: typeof String.prototype["lastIndexOf"] === "function". Actual: ' + (typeof String.prototype["lastIndexOf"] ));
if (typeof String.prototype.split !== "function")  throw new Test262Error('#19: typeof String.prototype.split === "function". Actual: ' + (typeof String.prototype.split ));
if (typeof String.prototype['split'] !== "function")  throw new Test262Error('#20: typeof String.prototype["split"] === "function". Actual: ' + (typeof String.prototype["split"] ));
if (typeof String.prototype.substring  !== "function")  throw new Test262Error('#21: typeof String.prototype.substring === "function". Actual: ' + (typeof String.prototype.substring ));
if (typeof String.prototype['substring']  !== "function")  throw new Test262Error('#22: typeof String.prototype["substring"] === "function". Actual: ' + (typeof String.prototype["substring"] ));
if (typeof String.prototype.toLowerCase !== "function")  throw new Test262Error('#23: typeof String.prototype.toLowerCase === "function". Actual: ' + (typeof String.prototype.toLowerCase ));
if (typeof String.prototype['toLowerCase'] !== "function")  throw new Test262Error('#24: typeof String.prototype["toLowerCase"] === "function". Actual: ' + (typeof String.prototype["toLowerCase"] ));
if (typeof String.prototype.toUpperCase !== "function")  throw new Test262Error('#25: typeof String.prototype.toUpperCase === "function". Actual: ' + (typeof String.prototype.toUpperCase ));
if (typeof String.prototype['toUpperCase'] !== "function")  throw new Test262Error('#26: typeof Array.prototype === "object". Actual: ' + (typeof Array.prototype ));
if (typeof String.prototype.length  !== "number")  throw new Test262Error('#27: typeof String.prototype.length === "number". Actual: ' + (typeof String.prototype.length ));
if (typeof String.prototype['length']  !== "number")  throw new Test262Error('#28: typeof String.prototype["length"] === "number". Actual: ' + (typeof String.prototype["length"] ));
