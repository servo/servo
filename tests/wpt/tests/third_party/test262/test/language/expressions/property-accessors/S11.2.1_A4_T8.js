// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: Check type of various properties
es5id: 11.2.1_A4_T8
description: Checking properties of the Math Object
---*/

//CHECK#1-52
if (typeof Math.E !== "number")  throw new Test262Error('#1: typeof Math.E === "number". Actual: ' + (typeof Math.E ));
if (typeof Math['E'] !== "number")  throw new Test262Error('#2: typeof Math["E"] === "number". Actual: ' + (typeof Math["E"] ));
if (typeof Math.LN10 !== "number")  throw new Test262Error('#3: typeof Math.LN10 === "number". Actual: ' + (typeof Math.LN10 ));
if (typeof Math['LN10'] !== "number")  throw new Test262Error('#4: typeof Math["LN10"] === "number". Actual: ' + (typeof Math["LN10"] ));
if (typeof Math.LN2 !== "number")  throw new Test262Error('#5: typeof Math.LN2 === "number". Actual: ' + (typeof Math.LN2 ));
if (typeof Math['LN2'] !== "number")  throw new Test262Error('#6: typeof Math["LN2"] === "number". Actual: ' + (typeof Math["LN2"] ));
if (typeof Math.LOG2E !== "number")  throw new Test262Error('#7: typeof Math.LOG2E === "number". Actual: ' + (typeof Math.LOG2E ));
if (typeof Math['LOG2E'] !== "number")  throw new Test262Error('#8: typeof Math["LOG2E"] === "number". Actual: ' + (typeof Math["LOG2E"] ));
if (typeof Math.LOG10E !== "number")  throw new Test262Error('#9: typeof Math.LOG10E === "number". Actual: ' + (typeof Math.LOG10E ));
if (typeof Math['LOG10E'] !== "number")  throw new Test262Error('#10: typeof Math["LOG10E"] === "number". Actual: ' + (typeof Math["LOG10E"] ));
if (typeof Math.PI !== "number")  throw new Test262Error('#11: typeof Math.PI === "number". Actual: ' + (typeof Math.PI ));
if (typeof Math['PI'] !== "number")  throw new Test262Error('#12: typeof Math["PI"] === "number". Actual: ' + (typeof Math["PI"] ));
if (typeof Math.SQRT1_2 !== "number")  throw new Test262Error('#13: typeof Math.SQRT1_2 === "number". Actual: ' + (typeof Math.SQRT1_2 ));
if (typeof Math['SQRT1_2'] !== "number")  throw new Test262Error('#14: typeof Math["SQRT1_2"] === "number". Actual: ' + (typeof Math["SQRT1_2"] ));
if (typeof Math.SQRT2 !== "number")  throw new Test262Error('#15: typeof Math.SQRT2 === "number". Actual: ' + (typeof Math.SQRT2 ));
if (typeof Math['SQRT2'] !== "number")  throw new Test262Error('#16: typeof Math["SQRT2"] === "number". Actual: ' + (typeof Math["SQRT2"] ));
if (typeof Math.abs !== "function")  throw new Test262Error('#17: typeof Math.abs === "function". Actual: ' + (typeof Math.abs ));
if (typeof Math['abs'] !== "function")  throw new Test262Error('#18: typeof Math["abs"] === "function". Actual: ' + (typeof Math["abs"] ));
if (typeof Math.acos !== "function")  throw new Test262Error('#19: typeof Math.acos === "function". Actual: ' + (typeof Math.acos ));
if (typeof Math['acos'] !== "function")  throw new Test262Error('#20: typeof Math["acos"] === "function". Actual: ' + (typeof Math["acos"] ));
if (typeof Math.asin !== "function")  throw new Test262Error('#21: typeof Math.asin === "function". Actual: ' + (typeof Math.asin ));
if (typeof Math['asin'] !== "function")  throw new Test262Error('#22: typeof Math["asin"] === "function". Actual: ' + (typeof Math["asin"] ));
if (typeof Math.atan !== "function")  throw new Test262Error('#23: typeof Math.atan === "function". Actual: ' + (typeof Math.atan ));
if (typeof Math['atan'] !== "function")  throw new Test262Error('#24: typeof Math["atan"] === "function". Actual: ' + (typeof Math["atan"] ));
if (typeof Math.atan2 !== "function")  throw new Test262Error('#25: typeof Math.atan2 === "function". Actual: ' + (typeof Math.atan2 ));
if (typeof Math['atan2'] !== "function")  throw new Test262Error('#26: typeof Math["atan2"] === "function". Actual: ' + (typeof Math["atan2"] ));
if (typeof Math.ceil !== "function")  throw new Test262Error('#27: typeof Math.ceil === "function". Actual: ' + (typeof Math.ceil ));
if (typeof Math['ceil'] !== "function")  throw new Test262Error('#28: typeof Math["ceil"] === "function". Actual: ' + (typeof Math["ceil"] ));
if (typeof Math.cos !== "function")  throw new Test262Error('#29: typeof Math.cos === "function". Actual: ' + (typeof Math.cos ));
if (typeof Math['cos'] !== "function")  throw new Test262Error('#30: typeof Math["cos"] === "function". Actual: ' + (typeof Math["cos"] ));
if (typeof Math.exp !== "function")  throw new Test262Error('#31: typeof Math.exp === "function". Actual: ' + (typeof Math.exp ));
if (typeof Math['exp'] !== "function")  throw new Test262Error('#32: typeof Math["exp"] === "function". Actual: ' + (typeof Math["exp"] ));
if (typeof Math.floor !== "function")  throw new Test262Error('#33: typeof Math.floor === "function". Actual: ' + (typeof Math.floor ));
if (typeof Math['floor'] !== "function")  throw new Test262Error('#34: typeof Math["floor"] === "function". Actual: ' + (typeof Math["floor"] ));
if (typeof Math.log !== "function")  throw new Test262Error('#35: typeof Math.log === "function". Actual: ' + (typeof Math.log ));
if (typeof Math['log'] !== "function")  throw new Test262Error('#36: typeof Math["log"] === "function". Actual: ' + (typeof Math["log"] ));
if (typeof Math.max !== "function")  throw new Test262Error('#37: typeof Math.max === "function". Actual: ' + (typeof Math.max ));
if (typeof Math['max'] !== "function")  throw new Test262Error('#38: typeof Math["max"] === "function". Actual: ' + (typeof Math["max"] ));
if (typeof Math.min !== "function")  throw new Test262Error('#39: typeof Math.min === "function". Actual: ' + (typeof Math.min ));
if (typeof Math['min'] !== "function")  throw new Test262Error('#40: typeof Math["min"] === "function". Actual: ' + (typeof Math["min"] ));
if (typeof Math.pow !== "function")  throw new Test262Error('#41: typeof Math.pow === "function". Actual: ' + (typeof Math.pow ));
if (typeof Math['pow'] !== "function")  throw new Test262Error('#42: typeof Math["pow"] === "function". Actual: ' + (typeof Math["pow"] ));
if (typeof Math.random !== "function")  throw new Test262Error('#43: typeof Math.random === "function". Actual: ' + (typeof Math.random ));
if (typeof Math['random'] !== "function")  throw new Test262Error('#44: typeof Math["random"] === "function". Actual: ' + (typeof Math["random"] ));
if (typeof Math.round !== "function")  throw new Test262Error('#45: typeof Math.round === "function". Actual: ' + (typeof Math.round ));
if (typeof Math['round'] !== "function")  throw new Test262Error('#46: typeof Math["round"] === "function". Actual: ' + (typeof Math["round"] ));
if (typeof Math.sin !== "function")  throw new Test262Error('#47: typeof Math.sin === "function". Actual: ' + (typeof Math.sin ));
if (typeof Math['sin'] !== "function")  throw new Test262Error('#48: typeof Math["sin"] === "function". Actual: ' + (typeof Math["sin"] ));
if (typeof Math.sqrt !== "function")  throw new Test262Error('#49: typeof Math.sqrt === "function". Actual: ' + (typeof Math.sqrt ));
if (typeof Math['sqrt'] !== "function")  throw new Test262Error('#50: typeof Math["sqrt"] === "function". Actual: ' + (typeof Math["sqrt"] ));
if (typeof Math.tan !== "function")  throw new Test262Error('#51: typeof Math.tan === "function". Actual: ' + (typeof Math.tan ));
if (typeof Math['tan'] !== "function")  throw new Test262Error('#52: typeof Math["tan"] === "function". Actual: ' + (typeof Math["tan"] ));
