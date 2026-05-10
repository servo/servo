/*
 * Any copyright is dedicated to the Public Domain.
 * http://creativecommons.org/licenses/publicdomain/
 */

/*---
flags:
  - noStrict
description: |
  Implement the ES5 algorithm for processing function statements
info: bugzilla.mozilla.org/show_bug.cgi?id=577325
esid: pending
---*/

var outer, desc;
var isInShell = !("Window" in this);

///////////////////////////////////////////////////
// Function definitions over accessor properties //
///////////////////////////////////////////////////

var getCalled, setCalled;

// configurable properties get blown away

getCalled = false, setCalled = false;
Object.defineProperty(this, "acc1",
                      {
                        get: function() { getCalled = true; throw "FAIL get 1"; },
                        set: function(v) { setCalled = true; throw "FAIL set 1 " + v; },
                        configurable: true,
                        enumerable: false
                      });

// does not throw
outer = undefined;
eval("function acc1() { throw 'FAIL redefined 1'; } outer = acc1;");
assert.sameValue(getCalled, false);
assert.sameValue(setCalled, false);
assert.sameValue(typeof acc1, "function");
assert.sameValue(acc1, outer);
desc = Object.getOwnPropertyDescriptor(this, "acc1");
assert.sameValue(desc.value, acc1);
assert.sameValue(desc.writable, true);
assert.sameValue(desc.enumerable, true);
assert.sameValue(desc.configurable, true);


getCalled = false, setCalled = false;
Object.defineProperty(this, "acc2",
                      {
                        get: function() { getCalled = true; throw "FAIL get 2"; },
                        set: function(v) { setCalled = true; throw "FAIL set 2 " + v; },
                        configurable: true,
                        enumerable: true
                      });

// does not throw
outer = undefined;
eval("function acc2() { throw 'FAIL redefined 2'; } outer = acc2;");
assert.sameValue(getCalled, false);
assert.sameValue(setCalled, false);
assert.sameValue(typeof acc2, "function");
assert.sameValue(acc2, outer);
desc = Object.getOwnPropertyDescriptor(this, "acc2");
assert.sameValue(desc.value, acc2);
assert.sameValue(desc.writable, true);
assert.sameValue(desc.enumerable, true);
assert.sameValue(desc.configurable, true);


// non-configurable properties produce a TypeError.  We only test this in shell,
// since defining non-configurable properties on Window instances throws.
if (isInShell) {
    getCalled = false, setCalled = false;
    Object.defineProperty(this, "acc3",
			  {
                              get: function() { getCalled = true; throw "FAIL get 3"; },
                              set: function(v) { setCalled = true; throw "FAIL set 3 " + v; },
                              configurable: false,
                              enumerable: true
			  });

    outer = undefined;
    try
    {
	eval("function acc3() { throw 'FAIL redefined 3'; }; outer = acc3");
	throw new Error("should have thrown trying to redefine global function " +
			"over a non-configurable, enumerable accessor");
    }
    catch (e)
    {
	assert.sameValue(e instanceof TypeError, true,
		 "global function definition, when that function would overwrite " +
		 "a non-configurable, enumerable accessor, must throw a TypeError " +
		 "per ES5+errata: " + e);
	desc = Object.getOwnPropertyDescriptor(this, "acc3");
	assert.sameValue(typeof desc.get, "function");
	assert.sameValue(typeof desc.set, "function");
	assert.sameValue(desc.enumerable, true);
	assert.sameValue(desc.configurable, false);
	assert.sameValue(outer, undefined);
	assert.sameValue(getCalled, false);
	assert.sameValue(setCalled, false);
    }


    getCalled = false, setCalled = false;
    Object.defineProperty(this, "acc4",
			  {
                              get: function() { getCalled = true; throw "FAIL get 4"; },
                              set: function(v) { setCalled = true; throw "FAIL set 4 " + v; },
                              configurable: false,
                              enumerable: false
			  });

    outer = undefined;
    try
    {
	eval("function acc4() { throw 'FAIL redefined 4'; }; outer = acc4");
	throw new Error("should have thrown trying to redefine global function " +
			"over a non-configurable, non-enumerable accessor");
    }
    catch (e)
    {
	assert.sameValue(e instanceof TypeError, true,
		 "global function definition, when that function would overwrite " +
		 "a non-configurable, non-enumerable accessor, must throw a " +
		 "TypeError per ES5+errata: " + e);
	desc = Object.getOwnPropertyDescriptor(this, "acc4");
	assert.sameValue(typeof desc.get, "function");
	assert.sameValue(typeof desc.set, "function");
	assert.sameValue(desc.enumerable, false);
	assert.sameValue(desc.configurable, false);
	assert.sameValue(outer, undefined);
	assert.sameValue(getCalled, false);
	assert.sameValue(setCalled, false);
    }
}


///////////////////////////////////////////////
// Function definitions over data properties //
///////////////////////////////////////////////


// configurable properties, regardless of other attributes, get blown away

Object.defineProperty(this, "data1",
                      {
                        configurable: true,
                        enumerable: true,
                        writable: true,
                        value: "data1"
                      });

outer = undefined;
eval("function data1() { return 'data1 function'; } outer = data1;");
assert.sameValue(typeof data1, "function");
assert.sameValue(data1, outer);
desc = Object.getOwnPropertyDescriptor(this, "data1");
assert.sameValue(desc.configurable, true);
assert.sameValue(desc.enumerable, true);
assert.sameValue(desc.writable, true);
assert.sameValue(desc.value, data1);


Object.defineProperty(this, "data2",
                      {
                        configurable: true,
                        enumerable: true,
                        writable: false,
                        value: "data2"
                      });

outer = undefined;
eval("function data2() { return 'data2 function'; } outer = data2;");
assert.sameValue(typeof data2, "function");
assert.sameValue(data2, outer);
desc = Object.getOwnPropertyDescriptor(this, "data2");
assert.sameValue(desc.configurable, true);
assert.sameValue(desc.enumerable, true);
assert.sameValue(desc.writable, true);
assert.sameValue(desc.value, data2);


Object.defineProperty(this, "data3",
                      {
                        configurable: true,
                        enumerable: false,
                        writable: true,
                        value: "data3"
                      });

outer = undefined;
eval("function data3() { return 'data3 function'; } outer = data3;");
assert.sameValue(typeof data3, "function");
assert.sameValue(data3, outer);
desc = Object.getOwnPropertyDescriptor(this, "data3");
assert.sameValue(desc.configurable, true);
assert.sameValue(desc.enumerable, true);
assert.sameValue(desc.writable, true);
assert.sameValue(desc.value, data3);


Object.defineProperty(this, "data4",
                      {
                        configurable: true,
                        enumerable: false,
                        writable: false,
                        value: "data4"
                      });

outer = undefined;
eval("function data4() { return 'data4 function'; } outer = data4;");
assert.sameValue(typeof data4, "function");
assert.sameValue(data4, outer);
desc = Object.getOwnPropertyDescriptor(this, "data4");
assert.sameValue(desc.value, data4);
assert.sameValue(desc.writable, true);
assert.sameValue(desc.enumerable, true);
assert.sameValue(desc.configurable, true);


// non-configurable data properties are trickier.  Again, we test these only in shell.

if (isInShell) {
    Object.defineProperty(this, "data5",
			  {
                              configurable: false,
                              enumerable: true,
                              writable: true,
                              value: "data5"
			  });

    outer = undefined;
    eval("function data5() { return 'data5 function'; } outer = data5;");
    assert.sameValue(typeof data5, "function");
    assert.sameValue(data5, outer);
    desc = Object.getOwnPropertyDescriptor(this, "data5");
    assert.sameValue(desc.configurable, false);
    assert.sameValue(desc.enumerable, true);
    assert.sameValue(desc.writable, true);
    assert.sameValue(desc.value, data5);


    Object.defineProperty(this, "data6",
			  {
                              configurable: false,
                              enumerable: true,
                              writable: false,
                              value: "data6"
			  });

    outer = undefined;
    try
    {
	eval("function data6() { return 'data6 function'; } outer = data6;");
	throw new Error("should have thrown trying to redefine global function " +
			"over a non-configurable, enumerable, non-writable accessor");
    }
    catch (e)
    {
	assert.sameValue(e instanceof TypeError, true,
		 "global function definition, when that function would overwrite " +
		 "a non-configurable, enumerable, non-writable data property, must " +
		 "throw a TypeError per ES5+errata: " + e);
	assert.sameValue(data6, "data6");
	assert.sameValue(outer, undefined);
	desc = Object.getOwnPropertyDescriptor(this, "data6");
	assert.sameValue(desc.configurable, false);
	assert.sameValue(desc.enumerable, true);
	assert.sameValue(desc.writable, false);
	assert.sameValue(desc.value, "data6");
    }


    Object.defineProperty(this, "data7",
			  {
                              configurable: false,
                              enumerable: false,
                              writable: true,
                              value: "data7"
			  });

    outer = undefined;
    try
    {
	eval("function data7() { return 'data7 function'; } outer = data7;");
	throw new Error("should have thrown trying to redefine global function " +
			"over a non-configurable, non-enumerable, writable data" +
			"property");
    }
    catch (e)
    {
	assert.sameValue(e instanceof TypeError, true,
		 "global function definition, when that function would overwrite " +
		 "a non-configurable, non-enumerable, writable data property, must " +
		 "throw a TypeError per ES5+errata: " + e);
	assert.sameValue(data7, "data7");
	assert.sameValue(outer, undefined);
	desc = Object.getOwnPropertyDescriptor(this, "data7");
	assert.sameValue(desc.configurable, false);
	assert.sameValue(desc.enumerable, false);
	assert.sameValue(desc.writable, true);
	assert.sameValue(desc.value, "data7");
    }


    Object.defineProperty(this, "data8",
			  {
                              configurable: false,
                              enumerable: false,
                              writable: false,
                              value: "data8"
			  });

    outer = undefined;
    try
    {
	eval("function data8() { return 'data8 function'; } outer = data8;");
	throw new Error("should have thrown trying to redefine global function " +
			"over a non-configurable, non-enumerable, non-writable data" +
			"property");
    }
    catch (e)
    {
	assert.sameValue(e instanceof TypeError, true,
		 "global function definition, when that function would overwrite " +
		 "a non-configurable, non-enumerable, non-writable data property, " +
		 "must throw a TypeError per ES5+errata: " + e);
	assert.sameValue(data8, "data8");
	assert.sameValue(outer, undefined);
	desc = Object.getOwnPropertyDescriptor(this, "data8");
	assert.sameValue(desc.configurable, false);
	assert.sameValue(desc.enumerable, false);
	assert.sameValue(desc.writable, false);
	assert.sameValue(desc.value, "data8");
    }
}
