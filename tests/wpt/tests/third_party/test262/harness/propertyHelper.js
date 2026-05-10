// Copyright (C) 2017 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
description: |
    Collection of functions used to safely verify the correctness of
    property descriptors.
defines:
  - verifyProperty
  - verifyCallableProperty
  - verifyEqualTo # deprecated
  - verifyWritable # deprecated
  - verifyNotWritable # deprecated
  - verifyEnumerable # deprecated
  - verifyNotEnumerable # deprecated
  - verifyConfigurable # deprecated
  - verifyNotConfigurable # deprecated
  - verifyPrimordialProperty
  - verifyPrimordialCallableProperty
---*/

// @ts-check

// Capture primordial functions and receiver-uncurried primordial methods that
// are used in verification but might be destroyed *by* that process itself.
var __isArray = Array.isArray;
var __defineProperty = Object.defineProperty;
var __getOwnPropertyDescriptor = Object.getOwnPropertyDescriptor;
var __getOwnPropertyNames = Object.getOwnPropertyNames;
var __join = Function.prototype.call.bind(Array.prototype.join);
var __push = Function.prototype.call.bind(Array.prototype.push);
var __hasOwnProperty = Function.prototype.call.bind(Object.prototype.hasOwnProperty);
var __propertyIsEnumerable = Function.prototype.call.bind(Object.prototype.propertyIsEnumerable);
var nonIndexNumericPropertyName = Math.pow(2, 32) - 1;

/**
 * @param {object} obj
 * @param {string|symbol} name
 * @param {PropertyDescriptor|undefined} desc
 * @param {object} [options]
 * @param {boolean} [options.restore] revert mutations from verifying writable/configurable
 */
function verifyProperty(obj, name, desc, options) {
  assert(
    arguments.length > 2,
    'verifyProperty should receive at least 3 arguments: obj, name, and descriptor'
  );

  var originalDesc = __getOwnPropertyDescriptor(obj, name);
  var nameStr = String(name);

  // Allows checking for undefined descriptor if it's explicitly given.
  if (desc === undefined) {
    assert.sameValue(
      originalDesc,
      undefined,
      "obj['" + nameStr + "'] descriptor should be undefined"
    );

    // desc and originalDesc are both undefined, problem solved;
    return true;
  }

  assert(
    __hasOwnProperty(obj, name),
    "obj should have an own property " + nameStr
  );

  assert.notSameValue(
    desc,
    null,
    "The desc argument should be an object or undefined, null"
  );

  assert.sameValue(
    typeof desc,
    "object",
    "The desc argument should be an object or undefined, " + String(desc)
  );

  var names = __getOwnPropertyNames(desc);
  for (var i = 0; i < names.length; i++) {
    assert(
      names[i] === "value" ||
        names[i] === "writable" ||
        names[i] === "enumerable" ||
        names[i] === "configurable" ||
        names[i] === "get" ||
        names[i] === "set",
      "Invalid descriptor field: " + names[i],
    );
  }

  var failures = [];

  if (__hasOwnProperty(desc, 'value')) {
    if (!isSameValue(desc.value, originalDesc.value)) {
      __push(failures, "obj['" + nameStr + "'] descriptor value should be " + desc.value);
    }
    if (!isSameValue(desc.value, obj[name])) {
      __push(failures, "obj['" + nameStr + "'] value should be " + desc.value);
    }
  }

  if (__hasOwnProperty(desc, 'enumerable') && desc.enumerable !== undefined) {
    if (desc.enumerable !== originalDesc.enumerable ||
        desc.enumerable !== isEnumerable(obj, name)) {
      __push(failures, "obj['" + nameStr + "'] descriptor should " + (desc.enumerable ? '' : 'not ') + "be enumerable");
    }
  }

  // Operations past this point are potentially destructive!

  if (__hasOwnProperty(desc, 'writable') && desc.writable !== undefined) {
    if (desc.writable !== originalDesc.writable ||
        desc.writable !== isWritable(obj, name)) {
      __push(failures, "obj['" + nameStr + "'] descriptor should " + (desc.writable ? '' : 'not ') + "be writable");
    }
  }

  if (__hasOwnProperty(desc, 'configurable') && desc.configurable !== undefined) {
    if (desc.configurable !== originalDesc.configurable ||
        desc.configurable !== isConfigurable(obj, name)) {
      __push(failures, "obj['" + nameStr + "'] descriptor should " + (desc.configurable ? '' : 'not ') + "be configurable");
    }
  }

  if (failures.length) {
    assert(false, __join(failures, '; '));
  }

  if (options && options.restore) {
    __defineProperty(obj, name, originalDesc);
  }

  return true;
}

function isConfigurable(obj, name) {
  try {
    delete obj[name];
  } catch (e) {
    if (!(e instanceof TypeError)) {
      throw new Test262Error("Expected TypeError, got " + e);
    }
  }
  return !__hasOwnProperty(obj, name);
}

function isEnumerable(obj, name) {
  var stringCheck = false;

  if (typeof name === "string") {
    for (var x in obj) {
      if (x === name) {
        stringCheck = true;
        break;
      }
    }
  } else {
    // skip it if name is not string, works for Symbol names.
    stringCheck = true;
  }

  return stringCheck && __hasOwnProperty(obj, name) && __propertyIsEnumerable(obj, name);
}

function isSameValue(a, b) {
  if (a === 0 && b === 0) return 1 / a === 1 / b;
  if (a !== a && b !== b) return true;

  return a === b;
}

function isWritable(obj, name, verifyProp, value) {
  var unlikelyValue = __isArray(obj) && name === "length" ?
    nonIndexNumericPropertyName :
    "unlikelyValue";
  var newValue = value || unlikelyValue;
  var hadValue = __hasOwnProperty(obj, name);
  var oldValue = obj[name];
  var writeSucceeded;

  if (arguments.length < 4 && newValue === oldValue) {
    newValue = newValue + "2";
  }

  try {
    obj[name] = newValue;
  } catch (e) {
    if (!(e instanceof TypeError)) {
      throw new Test262Error("Expected TypeError, got " + e);
    }
  }

  writeSucceeded = isSameValue(obj[verifyProp || name], newValue);

  // Revert the change only if it was successful (in other cases, reverting
  // is unnecessary and may trigger exceptions for certain property
  // configurations)
  if (writeSucceeded) {
    if (hadValue) {
      obj[name] = oldValue;
    } else {
      delete obj[name];
    }
  }

  return writeSucceeded;
}

/**
 * Verify that there is a function of specified name, length, and containing
 * descriptor associated with `obj[name]` and following the conventions for
 * built-in objects.
 *
 * @param {object} obj
 * @param {string|symbol} name
 * @param {string} [functionName] defaults to name for strings, `[${name.description}]` for symbols
 * @param {number} functionLength
 * @param {PropertyDescriptor} [desc] defaults to data property conventions (writable, non-enumerable, configurable)
 * @param {object} [options]
 * @param {boolean} [options.restore] revert mutations from verifying writable/configurable
 */
function verifyCallableProperty(obj, name, functionName, functionLength, desc, options) {
  var value = obj[name];

  assert.sameValue(typeof value, "function",
    "obj['" + String(name) + "'] descriptor should be a function");

  // Every other data property described in clauses 19 through 28 and in
  // Annex B.2 has the attributes { [[Writable]]: true, [[Enumerable]]: false,
  // [[Configurable]]: true } unless otherwise specified.
  // https://tc39.es/ecma262/multipage/ecmascript-standard-built-in-objects.html
  if (desc === undefined) {
    desc = {
      writable: true,
      enumerable: false,
      configurable: true,
      value: value
    };
  } else if (!__hasOwnProperty(desc, "value") && !__hasOwnProperty(desc, "get")) {
    desc.value = value;
  }

  verifyProperty(obj, name, desc, options);

  if (functionName === undefined) {
    if (typeof name === "symbol") {
      functionName = "[" + name.description + "]";
    } else {
      functionName = name;
    }
  }
  // Unless otherwise specified, the "name" property of a built-in function
  // object has the attributes { [[Writable]]: false, [[Enumerable]]: false,
  // [[Configurable]]: true }.
  // https://tc39.es/ecma262/multipage/ecmascript-standard-built-in-objects.html#sec-ecmascript-standard-built-in-objects
  // https://tc39.es/ecma262/multipage/ordinary-and-exotic-objects-behaviours.html#sec-setfunctionname
  verifyProperty(value, "name", {
    value: functionName,
    writable: false,
    enumerable: false,
    configurable: desc.configurable
  }, options);

  // Unless otherwise specified, the "length" property of a built-in function
  // object has the attributes { [[Writable]]: false, [[Enumerable]]: false,
  // [[Configurable]]: true }.
  // https://tc39.es/ecma262/multipage/ecmascript-standard-built-in-objects.html#sec-ecmascript-standard-built-in-objects
  // https://tc39.es/ecma262/multipage/ordinary-and-exotic-objects-behaviours.html#sec-setfunctionlength
  verifyProperty(value, "length", {
    value: functionLength,
    writable: false,
    enumerable: false,
    configurable: desc.configurable
  }, options);
}

/**
 * Deprecated; please use `verifyProperty` in new tests.
 */
function verifyEqualTo(obj, name, value) {
  if (!isSameValue(obj[name], value)) {
    throw new Test262Error("Expected obj[" + String(name) + "] to equal " + value +
           ", actually " + obj[name]);
  }
}

/**
 * Deprecated; please use `verifyProperty` in new tests.
 */
function verifyWritable(obj, name, verifyProp, value) {
  if (!verifyProp) {
    assert(__getOwnPropertyDescriptor(obj, name).writable,
         "Expected obj[" + String(name) + "] to have writable:true.");
  }
  if (!isWritable(obj, name, verifyProp, value)) {
    throw new Test262Error("Expected obj[" + String(name) + "] to be writable, but was not.");
  }
}

/**
 * Deprecated; please use `verifyProperty` in new tests.
 */
function verifyNotWritable(obj, name, verifyProp, value) {
  if (!verifyProp) {
    assert(!__getOwnPropertyDescriptor(obj, name).writable,
         "Expected obj[" + String(name) + "] to have writable:false.");
  }
  if (isWritable(obj, name, verifyProp)) {
    throw new Test262Error("Expected obj[" + String(name) + "] NOT to be writable, but was.");
  }
}

/**
 * Deprecated; please use `verifyProperty` in new tests.
 */
function verifyEnumerable(obj, name) {
  assert(__getOwnPropertyDescriptor(obj, name).enumerable,
       "Expected obj[" + String(name) + "] to have enumerable:true.");
  if (!isEnumerable(obj, name)) {
    throw new Test262Error("Expected obj[" + String(name) + "] to be enumerable, but was not.");
  }
}

/**
 * Deprecated; please use `verifyProperty` in new tests.
 */
function verifyNotEnumerable(obj, name) {
  assert(!__getOwnPropertyDescriptor(obj, name).enumerable,
       "Expected obj[" + String(name) + "] to have enumerable:false.");
  if (isEnumerable(obj, name)) {
    throw new Test262Error("Expected obj[" + String(name) + "] NOT to be enumerable, but was.");
  }
}

/**
 * Deprecated; please use `verifyProperty` in new tests.
 */
function verifyConfigurable(obj, name) {
  assert(__getOwnPropertyDescriptor(obj, name).configurable,
       "Expected obj[" + String(name) + "] to have configurable:true.");
  if (!isConfigurable(obj, name)) {
    throw new Test262Error("Expected obj[" + String(name) + "] to be configurable, but was not.");
  }
}

/**
 * Deprecated; please use `verifyProperty` in new tests.
 */
function verifyNotConfigurable(obj, name) {
  assert(!__getOwnPropertyDescriptor(obj, name).configurable,
       "Expected obj[" + String(name) + "] to have configurable:false.");
  if (isConfigurable(obj, name)) {
    throw new Test262Error("Expected obj[" + String(name) + "] NOT to be configurable, but was.");
  }
}

/**
 * Use this function to verify the properties of a primordial object.
 * For non-primordial objects, use verifyProperty.
 * See: https://github.com/tc39/how-we-work/blob/main/terminology.md#primordial
 */
var verifyPrimordialProperty = verifyProperty;

/**
 * Use this function to verify the primordial function-valued properties.
 * For non-primordial functions, use verifyCallableProperty.
 * See: https://github.com/tc39/how-we-work/blob/main/terminology.md#primordial
 */
var verifyPrimordialCallableProperty = verifyCallableProperty;
