// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    Properties of the object being enumerated may be deleted during
    enumeration
es5id: 12.6.4_A7_T2
description: >
    Checking "for (var VariableDeclarationNoIn in Expression)
    Statement" case
---*/

var __obj, __accum;

__obj = Object.create(null);
__obj.aa = 1;
__obj.ba = 2;
__obj.ca = 3;

__accum="";

for (var __key in __obj){

    erasator_T_1000(__obj,"b");
  
    __accum+=(__key+__obj[__key]);
	
}

assert(
    __accum === "aa1ca3" || __accum === "ca3aa1",
    "Unexpected value: '" + __accum + "'"
);

// erasator is the hash map terminator
function erasator_T_1000(hash_map, charactr){
    for (var key in hash_map){
        if (key.indexOf(charactr)===0) {
            delete hash_map[key];
        };
    }
}
