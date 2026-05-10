// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: Any separators are admitted between declaration chunks
es5id: 13_A16
description: Inserting separators between declaration chunks
---*/

function
x
(
)
{
}
;

x();

function                                                    y                                   (                                          )                                              {};

y();

function

z

(

)

{
    
}

;

z();

// The following function expression is expressed with literal unicode
// characters so that parsers may benefit from this test. The included code
// points are as follows:
//
// "function\u0009\u2029w(\u000C)\u00A0{\u000D}"

function	 w() {}

w();
