// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: Check type of various properties
es5id: 11.2.1_A4_T1
description: Checking properties of this object
---*/

//CHECK#1-32
if (typeof (this.NaN)  === "undefined")  throw new Test262Error('#1: typeof (this.NaN) !== "undefined"');
if (typeof this['NaN']  === "undefined")  throw new Test262Error('#2: typeof this["NaN"] !== "undefined"');
if (typeof this.Infinity  === "undefined")  throw new Test262Error('#3: typeof this.Infinity !== "undefined"');
if (typeof this['Infinity']  === "undefined")  throw new Test262Error('#4: typeof this["Infinity"] !== "undefined"');
if (typeof this.parseInt  === "undefined")  throw new Test262Error('#5: typeof this.parseInt !== "undefined"');
if (typeof this['parseInt'] === "undefined")  throw new Test262Error('#6: typeof this["parseInt"] !== "undefined"');
if (typeof this.parseFloat  === "undefined")  throw new Test262Error('#7: typeof this.parseFloat !== "undefined"');
if (typeof this['parseFloat'] === "undefined")  throw new Test262Error('#8: typeof this["parseFloat"] !== "undefined"');
if (typeof this.isNaN  === "undefined")  throw new Test262Error('#13: typeof this.isNaN !== "undefined"');
if (typeof this['isNaN'] === "undefined")  throw new Test262Error('#14: typeof this["isNaN"] !== "undefined"');
if (typeof this.isFinite  === "undefined")  throw new Test262Error('#15: typeof this.isFinite !== "undefined"');
if (typeof this['isFinite'] === "undefined")  throw new Test262Error('#16: typeof this["isFinite"] !== "undefined"');
if (typeof this.Object === "undefined")  throw new Test262Error('#17: typeof this.Object !== "undefined"');
if (typeof this['Object'] === "undefined")  throw new Test262Error('#18: typeof this["Object"] !== "undefined"');
if (typeof this.Number === "undefined")  throw new Test262Error('#19: typeof this.Number !== "undefined"');
if (typeof this['Number'] === "undefined")  throw new Test262Error('#20: typeof this["Number"] !== "undefined"');
if (typeof this.Function === "undefined")  throw new Test262Error('#21: typeof this.Function !== "undefined"');
if (typeof this['Function'] === "undefined")  throw new Test262Error('#22: typeof this["Function"] !== "undefined"');
if (typeof this.Array === "undefined")  throw new Test262Error('#23: typeof this.Array !== "undefined"');
if (typeof this['Array'] === "undefined")  throw new Test262Error('#24: typeof this["Array"] !== "undefined"');
if (typeof this.String === "undefined")  throw new Test262Error('#25: typeof this.String !== "undefined"');
if (typeof this['String'] === "undefined")  throw new Test262Error('#26: typeof this["String"] !== "undefined"');
if (typeof this.Boolean === "undefined")  throw new Test262Error('#27: typeof this.Boolean !== "undefined"');
if (typeof this['Boolean'] === "undefined")  throw new Test262Error('#28: typeof this["Boolean"] !== "undefined"');
if (typeof this.Date === "undefined")  throw new Test262Error('#29: typeof this.Date !== "undefined"');
if (typeof this['Date'] === "undefined")  throw new Test262Error('#30: typeof this["Date"] !== "undefined"');
if (typeof this.Math === "undefined")  throw new Test262Error('#31: typeof this.Math !== "undefined"');
if (typeof this['Math'] === "undefined")  throw new Test262Error('#32: typeof this["Math"] !== "undefined"');
