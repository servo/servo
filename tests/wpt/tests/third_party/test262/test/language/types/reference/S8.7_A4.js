// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    Object Modification Resulting in a New Object for Not a Self-Modified
    Object leads to loss of integrity
es5id: 8.7_A4
description: >
    Create a reference to the string, and Concatenate some new text
    onto the string object
---*/

//////////////////////////////////////////////////////////////////////////////
//CHECK#
// Set item equal to a new string object
var item = new String("test");
// itemRef now refers to the same string object
var itemRef = item;
// Concatenate some new text onto the string object
// NOTE: This creates a new object, and does not modify
// the original object.
item += "ing";
// The values of item and itemRef are NOT equal, as a whole
// new string object has been created
if( item == itemRef ){
  throw new Test262Error('#1: var item = new String("test"); var itemRef = item; item += "ing"; item != itemRef');
};
//
//////////////////////////////////////////////////////////////////////////////
