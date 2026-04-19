// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: Reference to Self-Modifying Object remain the integrity
es5id: 8.7_A2
description: Create a reference to the array, and change original array
---*/

//////////////////////////////////////////////////////////////////////////////
//CHECK#1
// Create an array of items
var items = new Array( "one", "two", "three" );
// Create a reference to the array of items
var itemsRef = items;
// Add an item to the original array
items.push( "four" );var itemsRef = items;
// The length of each array should be the same,
// since they both point to the same array object
if( itemsRef.length !== 4){
  throw new Test262Error('#1: var items = new Array( "one", "two", "three" ); var itemsRef = items; items.push( "four" );var itemsRef = items; itemsRef.length !== 4');
};
//
//////////////////////////////////////////////////////////////////////////////

//////////////////////////////////////////////////////////////////////////////
//CHECK#
// Create an array of items
var items = new Array( "one", "two", "three" );
// Create a reference to the array of items
var itemsRef = items;
// Add an item to the original array
items[1]="duo";
// The length of each array should be the same,
// since they both point to the same array object
if( itemsRef[1] !== "duo"){
  throw new Test262Error('#2: var items = new Array( "one", "two", "three" ); var itemsRef = items; items[1]="duo"; itemsRef[1] === "duo". Actual: ' + (itemsRef[1]));
};
//
//////////////////////////////////////////////////////////////////////////////
