// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: Function is a data
es5id: 13_A10
description: Using function as a property of an object
---*/

function __ziggy__func(){return "ziggy stardust"}

var __music_box={};

__music_box.ziggy = __ziggy__func;

//////////////////////////////////////////////////////////////////////////////
//CHECK#
if (typeof __music_box.ziggy !== "function") {
	throw new Test262Error('#1: typeof __music_box.ziggy === "function". Actual: typeof __music_box.ziggy ==='+typeof __music_box.ziggy);
}
//
//////////////////////////////////////////////////////////////////////////////

//////////////////////////////////////////////////////////////////////////////
//CHECK#2
if (__music_box.ziggy() !== "ziggy stardust") {
	throw new Test262Error('#2: __music_box.ziggy() === "ziggy stardust". Actual: __music_box.ziggy() ==='+__music_box.ziggy());
}
//
//////////////////////////////////////////////////////////////////////////////
