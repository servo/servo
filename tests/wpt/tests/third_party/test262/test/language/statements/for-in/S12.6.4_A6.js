// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    The production IterationStatement: "for (var VariableDeclarationNoIn in
    Expression) Statement"
es5id: 12.6.4_A6
description: >
    Using Object with custom prototype as an Expression is
    appropriate. The prototype is "{feat:2,hint:"protohint"}"
---*/

var __accum, key;

function FACTORY(){this.prop=1;this.hint="hinted"};

FACTORY.prototype = {feat:2,hint:"protohint"};

var __instance = new FACTORY;

__accum="";

for (key in __instance){
	__accum+=(key + __instance[key]);
}

//////////////////////////////////////////////////////////////////////////////
//CHECK#1
if (!((__accum.indexOf("prop1")!==-1)&&(__accum.indexOf("feat2")!==-1)&&(__accum.indexOf("hinthinted")!==-1))) {
	throw new Test262Error('#1: (__accum.indexOf("prop1")!==-1)&&(__accum.indexOf("feat2")!==-1)&&(__accum.indexOf("hinthinted")!==-1)');
}
//
//////////////////////////////////////////////////////////////////////////////

//////////////////////////////////////////////////////////////////////////////
//CHECK#2
if (__accum.indexOf("hintprotohint")!==-1) {
	throw new Test262Error('#2: __accum.indexOf("hintprotohint") === -1. Actual:  __accum.indexOf("hintprotohint") ==='+ __accum.indexOf("hintprotohint")  );
}
//
//////////////////////////////////////////////////////////////////////////////
