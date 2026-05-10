// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    If (Evaluate Statement).type is "break" and (Evaluate Statement).target
    is in the current label set, (normal, (Evaluate Statement), empty) is
    returned while evaluating a loop
es5id: 12.6.3_A12_T3
description: Trying to break non-existent label
negative:
  phase: parse
  type: SyntaxError
---*/

$DONOTEVALUATE();

__str="";

//////////////////////////////////////////////////////////////////////////////
//CHECK#4
outer:for(index=0;index<4;index+=1){
    nested:for(index_n=0;index_n<=index;index_n++){
        if(index*index_n >= 4)break nonexist;
        __str+=""+index+index_n;
    }
};
//
//////////////////////////////////////////////////////////////////////////////
