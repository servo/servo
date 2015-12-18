/*
 * Copyright 2008 The Closure Compiler Authors
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */

/**
 * @fileoverview ECMAScript 3 Built-Ins. This include common extensions so this
 * is actually ES3+Reality.
 * @externs
 * @author stevey@google.com (Steve Yegge)
 * @author nicksantos@google.com (Nick Santos)
 * @author arv@google.com (Erik Arvidsson)
 * @author johnlenz@google.com (John Lenz)
 */


// These built-ins are still needed for compilation.

/**
 * @constructor
 * @see http://developer.mozilla.org/en/Core_JavaScript_1.5_Reference/Functions_and_function_scope/arguments
 */
function Arguments() {}

/**
 * @type {Function}
 * @see http://developer.mozilla.org/En/Core_JavaScript_1.5_Reference/Functions_and_function_scope/arguments/callee
 */
Arguments.prototype.callee;

/**
 * Use the non-standard {@see Function.prototype.caller} property of a function
 * object instead.
 * @type {Function}
 * @see http://developer.mozilla.org/en/Core_JavaScript_1.5_Reference/Functions/arguments/caller
 * @deprecated
 */
Arguments.prototype.caller;

/**
 * @type {number}
 * @see http://developer.mozilla.org/En/Core_JavaScript_1.5_Reference/Functions_and_function_scope/arguments/length
 */
Arguments.prototype.length;

/**
 * @type {!Arguments}
 * @see http://developer.mozilla.org/en/Core_JavaScript_1.5_Reference/Functions_and_function_scope/arguments
 */
var arguments;

/**
 * @type {number}
 * @see http://developer.mozilla.org/en/Core_JavaScript_1.5_Reference/Global_Properties/Infinity
 * @const
 */
var Infinity;

/**
 * @type {number}
 * @see http://developer.mozilla.org/en/Core_JavaScript_1.5_Reference/Global_Properties/NaN
 * @const
 */
var NaN;

/**
 * @type {undefined}
 * @see http://developer.mozilla.org/en/Core_JavaScript_1.5_Reference/Global_Properties/undefined
 * @const
 */
var undefined;

/**
 * @param {string} uri
 * @return {string}
 * @nosideeffects
 * @see http://developer.mozilla.org/en/Core_JavaScript_1.5_Reference/Global_Functions/decodeURI
 */
function decodeURI(uri) {}

/**
 * @param {string} uri
 * @return {string}
 * @nosideeffects
 * @see http://developer.mozilla.org/en/Core_JavaScript_1.5_Reference/Global_Functions/decodeURIComponent
 */
function decodeURIComponent(uri) {}

/**
 * @param {string} uri
 * @return {string}
 * @nosideeffects
 * @see http://developer.mozilla.org/en/Core_JavaScript_1.5_Reference/Global_Functions/encodeURI
 */
function encodeURI(uri) {}

/**
 * @param {string} uri
 * @return {string}
 * @nosideeffects
 * @see http://developer.mozilla.org/en/Core_JavaScript_1.5_Reference/Global_Functions/encodeURIComponent
 */
function encodeURIComponent(uri) {}

/**
 * Should only be used in browsers where encode/decodeURIComponent
 * are not present, as the latter handle fancy Unicode characters.
 * @param {string} str
 * @return {string}
 * @nosideeffects
 * @see https://developer.mozilla.org/en/Core_JavaScript_1.5_Guide/Predefined_Functions/escape_and_unescape_Functions
 */
function escape(str) {}

/**
 * Should only be used in browsers where encode/decodeURIComponent
 * are not present, as the latter handle fancy Unicode characters.
 * @param {string} str
 * @return {string}
 * @nosideeffects
 * @see https://developer.mozilla.org/en/Core_JavaScript_1.5_Guide/Predefined_Functions/escape_and_unescape_Functions
 */
function unescape(str) {}

/**
 * @param {*} num
 * @return {boolean}
 * @nosideeffects
 * @see http://developer.mozilla.org/en/Core_JavaScript_1.5_Reference/Global_Functions/isFinite
 */
function isFinite(num) {}

/**
 * @param {*} num
 * @return {boolean}
 * @nosideeffects
 * @see http://developer.mozilla.org/en/Core_JavaScript_1.5_Reference/Global_Functions/isNaN
 */
function isNaN(num) {}

/**
 * @param {*} num
 * @return {number}
 * @nosideeffects
 * @see http://developer.mozilla.org/en/Core_JavaScript_1.5_Reference/Global_Functions/parseFloat
 */
function parseFloat(num) {}

/**
 * Parse an integer. Use of {@code parseInt} without {@code base} is strictly
 * banned in Google. If you really want to parse octal or hex based on the
 * leader, then pass {@code undefined} as the base.
 *
 * @param {*} num
 * @param {number|undefined} base
 * @return {number}
 * @nosideeffects
 * @see http://developer.mozilla.org/en/Core_JavaScript_1.5_Reference/Global_Functions/parseInt
 */
function parseInt(num, base) {}

/**
 * @param {string} code
 * @return {*}
 * @see http://developer.mozilla.org/en/Core_JavaScript_1.5_Reference/Global_Functions/eval
 */
function eval(code) {}

/**
 * @constructor
 * @param {*=} opt_value
 * @return {!Object}
 * @nosideeffects
 * @see http://developer.mozilla.org/en/Core_JavaScript_1.5_Reference/Global_Objects/Object
 */
function Object(opt_value) {}

/**
 * The constructor of the current object.
 * @type {Function}
 * @see http://developer.mozilla.org/en/Core_JavaScript_1.5_Reference/Global_Objects/Object/constructor
 */
Object.prototype.constructor = function() {};

/**
 * Binds an object's property to a function to be called when that property is
 * looked up.
 * Mozilla-only.
 *
 * @param {string} sprop
 * @param {Function} fun
 * @modifies {this}
 * @see http://developer.mozilla.org/en/Core_JavaScript_1.5_Reference/Global_Objects/Object/defineGetter
 */
Object.prototype.__defineGetter__ = function(sprop, fun) {};

/**
 * Binds an object's property to a function to be called when an attempt is made
 * to set that property.
 * Mozilla-only.
 *
 * @param {string} sprop
 * @param {Function} fun
 * @modifies {this}
 * @see http://developer.mozilla.org/en/Core_JavaScript_1.5_Reference/Global_Objects/Object/defineSetter
 */
Object.prototype.__defineSetter__ = function(sprop, fun) {};

/**
 * Returns whether the object has a property with the specified name.
 *
 * @param {*} propertyName Implicitly cast to a string.
 * @return {boolean}
 * @nosideeffects
 * @see http://developer.mozilla.org/en/Core_JavaScript_1.5_Reference/Global_Objects/Object/hasOwnProperty
 */
Object.prototype.hasOwnProperty = function(propertyName) {};

/**
 * Returns whether an object exists in another object's prototype chain.
 *
 * @param {Object} other
 * @return {boolean}
 * @nosideeffects
 * @see http://developer.mozilla.org/en/Core_JavaScript_1.5_Reference/Global_Objects/Object/isPrototypeOf
 */
Object.prototype.isPrototypeOf = function(other) {};

/**
 * Return the function bound as a getter to the specified property.
 * Mozilla-only.
 *
 * @param {string} sprop a string containing the name of the property whose
 * getter should be returned
 * @return {Function}
 * @nosideeffects
 * @see http://developer.mozilla.org/en/Core_JavaScript_1.5_Reference/Global_Objects/Object/lookupGetter
 */
Object.prototype.__lookupGetter__ = function(sprop) {};

/**
 * Return the function bound as a setter to the specified property.
 * Mozilla-only.
 *
 * @param {string} sprop a string containing the name of the property whose
 *     setter should be returned.
 * @return {Function}
 * @nosideeffects
 * @see http://developer.mozilla.org/en/Core_JavaScript_1.5_Reference/Global_Objects/Object/lookupSetter
 */
Object.prototype.__lookupSetter__ = function(sprop) {};

/**
 * Executes a function when a non-existent method is called on an object.
 * Mozilla-only.
 *
 * @param {Function} fun
 * @return {*}
 * @see http://developer.mozilla.org/en/Core_JavaScript_1.5_Reference/Global_Objects/Object/noSuchMethod
 */
Object.prototype.__noSuchMethod__ = function(fun) {};

/**
 * Points to an object's context.  For top-level objects, this is the e.g. window.
 * Mozilla-only.
 *
 * @type {Object}
 * @deprecated
 * @see http://developer.mozilla.org/en/Core_JavaScript_1.5_Reference/Global_Objects/Object/parent
 */
Object.prototype.__parent__;

/**
 * Points to the object which was used as prototype when the object was instantiated.
 * Mozilla-only.
 *
 * Will be null on Object.prototype.
 *
 * @type {Object}
 * @see http://developer.mozilla.org/en/Core_JavaScript_1.5_Reference/Global_Objects/Object/proto
 */
Object.prototype.__proto__;

/**
 * Determine whether the specified property in an object can be enumerated by a
 * for..in loop, with the exception of properties inherited through the
 * prototype chain.
 *
 * @param {string} propertyName
 * @return {boolean}
 * @nosideeffects
 * @see http://developer.mozilla.org/en/Core_JavaScript_1.5_Reference/Global_Objects/Object/propertyIsEnumerable
 */
Object.prototype.propertyIsEnumerable = function(propertyName) {};

/**
 * Returns a localized string representing the object.
 * @return {string}
 * @nosideeffects
 * @see http://developer.mozilla.org/en/Core_JavaScript_1.5_Reference/Global_Objects/Object/toLocaleString
 */
Object.prototype.toLocaleString = function() {};

/**
 * Returns a string representing the source code of the object.
 * Mozilla-only.
 * @return {string}
 * @nosideeffects
 * @see http://developer.mozilla.org/en/Core_JavaScript_1.5_Reference/Global_Objects/Object/toSource
 */
Object.prototype.toSource = function() {};

/**
 * Returns a string representing the object.
 * @this {*}
 * @return {string}
 * @nosideeffects
 * @see http://developer.mozilla.org/en/Core_JavaScript_1.5_Reference/Global_Objects/Object/toString
 */
Object.prototype.toString = function() {};

/**
 * Removes a watchpoint set with the {@see Object.prototype.watch} method.
 * Mozilla-only.
 * @param {string} prop The name of a property of the object.
 * @see http://developer.mozilla.org/en/Core_JavaScript_1.5_Reference/Global_Objects/Object/unwatch
 */
Object.prototype.unwatch = function(prop) {};

/**
 * Returns the object's {@code this} value.
 * @return {*}
 * @nosideeffects
 * @see http://developer.mozilla.org/en/Core_JavaScript_1.5_Reference/Global_Objects/Object/valueOf
 */
Object.prototype.valueOf = function() {};

/**
 * Sets a watchpoint method.
 * Mozilla-only.
 * @param {string} prop The name of a property of the object.
 * @param {Function} handler A function to call.
 * @see http://developer.mozilla.org/en/Core_JavaScript_1.5_Reference/Global_Objects/Object/watch
 */
Object.prototype.watch = function(prop, handler) {};


/**
 * @constructor
 * @param {...*} var_args
 * @nosideeffects
 * @throws {Error}
 * @see http://developer.mozilla.org/en/Core_JavaScript_1.5_Reference/Global_Objects/Function
 */
function Function(var_args) {}

/**
 * @param {...*} var_args
 * @return {*}
 * @see http://developer.mozilla.org/en/Core_JavaScript_1.5_Reference/Global_Objects/Function/call
 */
Function.prototype.call = function(var_args) {};

/**
 * @param {...*} var_args
 * @return {*}
 * @see http://developer.mozilla.org/en/Core_JavaScript_1.5_Reference/Global_Objects/Function/apply
 */
Function.prototype.apply = function(var_args) {};

Function.prototype.arguments;

/**
 * @type {number}
 * @deprecated
 * @see http://developer.mozilla.org/en/Core_JavaScript_1.5_Reference/Global_Objects/Function/arity
 */
Function.prototype.arity;

/**
 * Nonstandard; Mozilla and JScript only.
 * @type {Function}
 * @see http://developer.mozilla.org/en/Core_JavaScript_1.5_Reference/Global_Objects/Function/caller
 */
Function.prototype.caller;

/**
 * Nonstandard.
 * @type {?}
 * @see http://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Function/displayName
 */
Function.prototype.displayName;

/**
 * Expected number of arguments.
 * @type {number}
 * @see http://developer.mozilla.org/en/Core_JavaScript_1.5_Reference/Global_Objects/Function/length
 */
Function.prototype.length;

/**
 * @type {string}
 * @see http://developer.mozilla.org/en/Core_JavaScript_1.5_Reference/Global_Objects/Function/name
 */
Function.prototype.name;

/**
 * @this {Function}
 * @return {string}
 * @nosideeffects
 * @override
 */
Function.prototype.toString = function() {};


/**
 * @constructor
 * @param {...*} var_args
 * @return {!Array.<?>}
 * @nosideeffects
 * @template T
 * @see http://developer.mozilla.org/en/Core_JavaScript_1.5_Reference/Global_Objects/Array
 */
function Array(var_args) {}

// Functions:

/**
 * Returns a new array comprised of this array joined with other array(s)
 * and/or value(s).
 *
 * @param {...*} var_args
 * @return {!Array.<?>}
 * @this {*}
 * @nosideeffects
 * @see http://developer.mozilla.org/en/Core_JavaScript_1.5_Reference/Global_Objects/Array/concat
 */
Array.prototype.concat = function(var_args) {};

/**
 * Joins all elements of an array into a string.
 *
 * @param {*=} opt_separator Specifies a string to separate each element of the
 *     array. The separator is converted to a string if necessary. If omitted,
 *     the array elements are separated with a comma.
 * @return {string}
 * @this {{length: number}|string}
 * @nosideeffects
 * @see http://developer.mozilla.org/en/Core_JavaScript_1.5_Reference/Global_Objects/Array/join
 */
Array.prototype.join = function(opt_separator) {};

/**
 * Removes the last element from an array and returns that element.
 *
 * @return {T}
 * @this {{length: number}|Array.<T>}
 * @modifies {this}
 * @template T
 * @see http://developer.mozilla.org/en/Core_JavaScript_1.5_Reference/Global_Objects/Array/pop
 */
Array.prototype.pop = function() {};

/**
 * Mutates an array by appending the given elements and returning the new
 * length of the array.
 *
 * @param {...T} var_args
 * @return {number} The new length of the array.
 * @this {{length: number}|Array.<T>}
 * @template T
 * @modifies {this}
 * @see http://developer.mozilla.org/en/Core_JavaScript_1.5_Reference/Global_Objects/Array/push
 */
Array.prototype.push = function(var_args) {};

/**
 * Transposes the elements of an array in place: the first array element becomes the
 * last and the last becomes the first.
 *
 * @this {{length: number}}
 * @modifies {this}
 * @see http://developer.mozilla.org/en/Core_JavaScript_1.5_Reference/Global_Objects/Array/reverse
 */
Array.prototype.reverse = function() {};

/**
 * Removes the first element from an array and returns that element. This
 * method changes the length of the array.
 *
 * @this {{length: number}|Array.<T>}
 * @modifies {this}
 * @return {T}
 * @template T
 * @see http://developer.mozilla.org/en/Core_JavaScript_1.5_Reference/Global_Objects/Array/shift
 */
Array.prototype.shift = function() {};

/**
 * Extracts a section of an array and returns a new array.
 *
 * @param {*=} opt_begin Zero-based index at which to begin extraction.  A
 *     non-number type will be auto-cast by the browser to a number.
 * @param {*=} opt_end Zero-based index at which to end extraction.  slice
 *     extracts up to but not including end.
 * @return {!Array.<T>}
 * @this {{length: number}|Array.<T>|string}
 * @template T
 * @nosideeffects
 * @see http://developer.mozilla.org/en/Core_JavaScript_1.5_Reference/Global_Objects/Array/slice
 */
Array.prototype.slice = function(opt_begin, opt_end) {};

/**
 * Sorts the elements of an array in place.
 *
 * @param {function(T,T):number=} opt_compareFunction Specifies a function that
 *     defines the sort order.
 * @this {{length: number}|Array.<T>}
 * @template T
 * @see http://developer.mozilla.org/en/Core_JavaScript_1.5_Reference/Global_Objects/Array/sort
 */
Array.prototype.sort = function(opt_compareFunction) {};

/**
 * Changes the content of an array, adding new elements while removing old
 * elements.
 *
 * @param {*=} opt_index Index at which to start changing the array. If negative,
 *     will begin that many elements from the end.  A non-number type will be
 *     auto-cast by the browser to a number.
 * @param {*=} opt_howMany An integer indicating the number of old array elements
 *     to remove.
 * @param {...T} var_args
 * @return {!Array.<T>}
 * @this {{length: number}|Array.<T>}
 * @modifies {this}
 * @template T
 * @see http://developer.mozilla.org/en/Core_JavaScript_1.5_Reference/Global_Objects/Array/splice
 */
Array.prototype.splice = function(opt_index, opt_howMany, var_args) {};

/**
 * @return {string}
 * @this {Object}
 * @nosideeffects
 * @see http://developer.mozilla.org/en/Core_JavaScript_1.5_Reference/Global_Objects/Array/toSource
 */
Array.prototype.toSource;

/**
 * @this {Array.<?>}
 * @return {string}
 * @nosideeffects
 * @override
 */
Array.prototype.toString = function() {};

/**
 * Adds one or more elements to the beginning of an array and returns the new
 * length of the array.
 *
 * @param {...*} var_args
 * @return {number} The new length of the array
 * @this {{length: number}}
 * @modifies {this}
 * @see http://developer.mozilla.org/en/Core_JavaScript_1.5_Reference/Global_Objects/Array/unshift
 */
Array.prototype.unshift = function(var_args) {};

/**
 * Apply a function simultaneously against two values of the array (from
 * left-to-right) as to reduce it to a single value.
 *
 * @param {?function(?, T, number, !Array.<T>) : R} callback
 * @param {*=} opt_initialValue
 * @return {R}
 * @this {{length: number}|Array.<T>|string}
 * @template T,R
 * @see http://developer.mozilla.org/en/Core_JavaScript_1.5_Reference/Global_Objects/Array/reduce
 */
Array.prototype.reduce = function(callback, opt_initialValue) {};

/**
 * Apply a function simultaneously against two values of the array (from
 * right-to-left) as to reduce it to a single value.
 *
 * @param {?function(?, T, number, !Array.<T>) : R} callback
 * @param {*=} opt_initialValue
 * @return {R}
 * @this {{length: number}|Array.<T>|string}
 * @template T,R
 * @see http://developer.mozilla.org/en/Core_JavaScript_1.5_Reference/Global_Objects/Array/reduceRight
 */
Array.prototype.reduceRight = function(callback, opt_initialValue) {};

/**
 * Available in ECMAScript 5, Mozilla 1.6+.
 * @param {?function(this:S, T, number, !Array.<T>): ?} callback
 * @param {S=} opt_thisobj
 * @return {boolean}
 * @this {{length: number}|Array.<T>|string}
 * @template T,S
 * @see http://developer.mozilla.org/en/Core_JavaScript_1.5_Reference/Global_Objects/Array/every
 */
Array.prototype.every = function(callback, opt_thisobj) {};

/**
 * Available in ECMAScript 5, Mozilla 1.6+.
 * @param {?function(this:S, T, number, !Array.<T>): ?} callback
 * @param {S=} opt_thisobj
 * @return {!Array.<T>}
 * @this {{length: number}|Array.<T>|string}
 * @template T,S
 * @see http://developer.mozilla.org/en/Core_JavaScript_1.5_Reference/Global_Objects/Array/filter
 */
Array.prototype.filter = function(callback, opt_thisobj) {};

/**
 * Available in ECMAScript 5, Mozilla 1.6+.
 * @param {?function(this:S, T, number, !Array.<T>): ?} callback
 * @param {S=} opt_thisobj
 * @this {{length: number}|Array.<T>|string}
 * @template T,S
 * @see http://developer.mozilla.org/en/Core_JavaScript_1.5_Reference/Global_Objects/Array/forEach
 */
Array.prototype.forEach = function(callback, opt_thisobj) {};

/**
 * Available in ECMAScript 5, Mozilla 1.6+.
 * @param {T} obj
 * @param {number=} opt_fromIndex
 * @return {number}
 * @this {{length: number}|Array.<T>|string}
 * @nosideeffects
 * @template T
 * @see http://developer.mozilla.org/en/Core_JavaScript_1.5_Reference/Global_Objects/Array/indexOf
 */
Array.prototype.indexOf = function(obj, opt_fromIndex) {};

/**
 * Available in ECMAScript 5, Mozilla 1.6+.
 * @param {T} obj
 * @param {number=} opt_fromIndex
 * @return {number}
 * @this {{length: number}|Array.<T>|string}
 * @nosideeffects
 * @template T
 * @see http://developer.mozilla.org/en/Core_JavaScript_1.5_Reference/Global_Objects/Array/lastIndexOf
 */
Array.prototype.lastIndexOf = function(obj, opt_fromIndex) {};

/**
 * Available in ECMAScript 5, Mozilla 1.6+.
 * @param {?function(this:S, T, number, !Array.<T>): R} callback
 * @param {S=} opt_thisobj
 * @return {!Array.<R>}
 * @this {{length: number}|Array.<T>|string}
 * @template T,S,R
 * @see http://developer.mozilla.org/en/Core_JavaScript_1.5_Reference/Global_Objects/Array/map
 */
Array.prototype.map = function(callback, opt_thisobj) {};

/**
 * Available in ECMAScript 5, Mozilla 1.6+.
 * @param {?function(this:S, T, number, !Array.<T>): ?} callback
 * @param {S=} opt_thisobj
 * @return {boolean}
 * @this {{length: number}|Array.<T>|string}
 * @template T,S
 * @see http://developer.mozilla.org/en/Core_JavaScript_1.5_Reference/Global_Objects/Array/some
 */
Array.prototype.some = function(callback, opt_thisobj) {};

/**
 * @type {number}
 * @see http://developer.mozilla.org/en/Core_JavaScript_1.5_Reference/Global_Objects/Array/index
 */
Array.prototype.index;

/**
 * @type {?string}
 * @see http://developer.mozilla.org/en/Core_JavaScript_1.5_Reference/Global_Objects/Array/input
 */
Array.prototype.input;

/**
 * @type {number}
 * @see http://developer.mozilla.org/en/Core_JavaScript_1.5_Reference/Global_Objects/Array/length
 */
Array.prototype.length;

/**
 * @param {{length: number}|Array.<T>} arr
 * @param {?function(this:S, T, number, ?) : ?} callback
 * @param {S=} opt_context
 * @return {boolean}
 * @template T,S
 */
Array.every = function(arr, callback, opt_context) {};

/**
 * @param {{length: number}|Array.<T>} arr
 * @param {?function(this:S, T, number, ?) : ?} callback
 * @param {S=} opt_context
 * @return {!Array.<T>}
 * @template T,S
 */
Array.filter = function(arr, callback, opt_context) {};

/**
 * @param {{length: number}|Array.<T>} arr
 * @param {?function(this:S, T, number, ?) : ?} callback
 * @param {S=} opt_context
 * @template T,S
 */
Array.forEach = function(arr, callback, opt_context) {};

/**
 * Mozilla 1.6+ only.
 * @param {{length: number}|Array.<T>} arr
 * @param {T} obj
 * @param {number=} opt_fromIndex
 * @return {number}
 * @template T
 * @nosideeffects
 * @see http://developer.mozilla.org/en/Core_JavaScript_1.5_Reference/Global_Objects/Array/indexOf
 */
Array.indexOf = function(arr, obj, opt_fromIndex) {};

/**
 * Mozilla 1.6+ only.
 * @param {{length: number}|Array.<T>} arr
 * @param {T} obj
 * @param {number=} opt_fromIndex
 * @return {number}
 * @template T
 * @nosideeffects
 * @see http://developer.mozilla.org/en/Core_JavaScript_1.5_Reference/Global_Objects/Array/lastIndexOf
 */
Array.lastIndexOf = function(arr, obj, opt_fromIndex) {};

/**
 * @param {{length: number}|Array.<T>} arr
 * @param {?function(this:S, T, number, !Array.<T>): R} callback
 * @param {S=} opt_context
 * @return {!Array.<R>}
 * @template T,S,R
 */
Array.map = function(arr, callback, opt_context) {};

/**
 * @param {{length: number}|Array.<T>} arr
 * @param {?function(this:S, T, number, ?) : ?} callback
 * @param {S=} opt_context
 * @return {boolean}
 * @template T,S
 */
Array.some = function(arr, callback, opt_context) {};

/**
 * Introduced in 1.8.5.
 * @param {*} arr
 * @return {boolean}
 * @see http://developer.mozilla.org/en/JavaScript/Reference/Global_Objects/Array/isArray
 */
Array.isArray = function(arr) {};

/**
 * @constructor
 * @param {*=} opt_value
 * @return {boolean}
 * @nosideeffects
 * @see http://developer.mozilla.org/en/Core_JavaScript_1.5_Reference/Global_Objects/Boolean
 */
function Boolean(opt_value) {}

/**
 * @return {string}
 * @nosideeffects
 * @see http://developer.mozilla.org/en/Core_JavaScript_1.5_Reference/Global_Objects/Boolean/toSource
 * @override
 */
Boolean.prototype.toSource = function() {};

/**
 * @this {boolean|Boolean}
 * @return {string}
 * @nosideeffects
 * @override
 */
Boolean.prototype.toString = function() {};

/**
 * @constructor
 * @param {*=} opt_value
 * @return {number}
 * @nosideeffects
 * @see http://developer.mozilla.org/en/Core_JavaScript_1.5_Reference/Global_Objects/Number
 */
function Number(opt_value) {}

/**
 * @this {Number|number}
 * @param {number=} opt_fractionDigits
 * @return {string}
 * @nosideeffects
 * @see http://developer.mozilla.org/en/Core_JavaScript_1.5_Reference/Global_Objects/Number/toExponential
 */
Number.prototype.toExponential = function(opt_fractionDigits) {};

/**
 * @this {Number|number}
 * @param {*=} opt_digits
 * @return {string}
 * @nosideeffects
 * @see http://developer.mozilla.org/en/Core_JavaScript_1.5_Reference/Global_Objects/Number/toFixed
 */
Number.prototype.toFixed = function(opt_digits) {};

/**
 * @this {Number|number}
 * @param {number=} opt_precision
 * @return {string}
 * @nosideeffects
 * @see http://developer.mozilla.org/en/Core_JavaScript_1.5_Reference/Global_Objects/Number/toPrecision
 */
Number.prototype.toPrecision = function(opt_precision) {};

/**
 * Returns a string representing the number.
 * @this {Number|number}
 * @param {(number|Number)=} opt_radix An optional radix.
 * @return {string}
 * @nosideeffects
 * @see http://developer.mozilla.org/en/Core_JavaScript_1.5_Reference/Global_Objects/Number/toString
 * @override
 */
Number.prototype.toString = function(opt_radix) {};

// Properties.
/**
 * @type {number}
 * @see http://developer.mozilla.org/en/Core_JavaScript_1.5_Reference/Global_Objects/Number/MAX_VALUE
 */
Number.MAX_VALUE;

/**
 * @type {number}
 * @see http://developer.mozilla.org/en/Core_JavaScript_1.5_Reference/Global_Objects/Number/MIN_VALUE
 */
Number.MIN_VALUE;

/**
 * @type {number}
 * @see http://developer.mozilla.org/en/Core_JavaScript_1.5_Reference/Global_Objects/Number/NaN
 */
Number.NaN;

/**
 * @type {number}
 * @see http://developer.mozilla.org/en/Core_JavaScript_1.5_Reference/Global_Objects/Number/NEGATIVE_INFINITY
 */
Number.NEGATIVE_INFINITY;

/**
 * @type {number}
 * @see http://developer.mozilla.org/en/Core_JavaScript_1.5_Reference/Global_Objects/Number/POSITIVE_INFINITY
 */
Number.POSITIVE_INFINITY;


/**
 * @const
 * @see http://developer.mozilla.org/en/Core_JavaScript_1.5_Reference/Global_Objects/Math
 */
var Math = {};

/**
 * @param {*} x
 * @return {number}
 * @nosideeffects
 * @see http://developer.mozilla.org/en/Core_JavaScript_1.5_Reference/Global_Objects/Math/abs
 */
Math.abs = function(x) {};

/**
 * @param {*} x
 * @return {number}
 * @nosideeffects
 * @see http://developer.mozilla.org/en/Core_JavaScript_1.5_Reference/Global_Objects/Math/acos
 */
Math.acos = function(x) {};

/**
 * @param {*} x
 * @return {number}
 * @nosideeffects
 * @see http://developer.mozilla.org/en/Core_JavaScript_1.5_Reference/Global_Objects/Math/asin
 */
Math.asin = function(x) {};

/**
 * @param {*} x
 * @return {number}
 * @nosideeffects
 * @see http://developer.mozilla.org/en/Core_JavaScript_1.5_Reference/Global_Objects/Math/atan
 */
Math.atan = function(x) {};

/**
 * @param {*} y
 * @param {*} x
 * @return {number}
 * @nosideeffects
 * @see http://developer.mozilla.org/en/Core_JavaScript_1.5_Reference/Global_Objects/Math/atan2
 */
Math.atan2 = function(y, x) {};

/**
 * @param {*} x
 * @return {number}
 * @nosideeffects
 * @see http://developer.mozilla.org/en/Core_JavaScript_1.5_Reference/Global_Objects/Math/ceil
 */
Math.ceil = function(x) {};

/**
 * @param {*} x
 * @return {number}
 * @nosideeffects
 * @see http://developer.mozilla.org/en/Core_JavaScript_1.5_Reference/Global_Objects/Math/cos
 */
Math.cos = function(x) {};

/**
 * @param {*} x
 * @return {number}
 * @nosideeffects
 * @see http://developer.mozilla.org/en/Core_JavaScript_1.5_Reference/Global_Objects/Math/exp
 */
Math.exp = function(x) {};

/**
 * @param {*} x
 * @return {number}
 * @nosideeffects
 * @see http://developer.mozilla.org/en/Core_JavaScript_1.5_Reference/Global_Objects/Math/floor
 */
Math.floor = function(x) {};

/**
 * @param {*} x
 * @return {number}
 * @nosideeffects
 * @see http://developer.mozilla.org/en/Core_JavaScript_1.5_Reference/Global_Objects/Math/log
 */
Math.log = function(x) {};

/**
 * @param {...*} var_args
 * @return {number}
 * @nosideeffects
 * @see http://developer.mozilla.org/en/Core_JavaScript_1.5_Reference/Global_Objects/Math/max
 */
Math.max = function(var_args) {};

/**
 * @param {...*} var_args
 * @return {number}
 * @nosideeffects
 * @see http://developer.mozilla.org/en/Core_JavaScript_1.5_Reference/Global_Objects/Math/min
 */
Math.min = function(var_args) {};

/**
 * @param {*} x
 * @param {*} y
 * @return {number}
 * @nosideeffects
 * @see http://developer.mozilla.org/en/Core_JavaScript_1.5_Reference/Global_Objects/Math/pow
 */
Math.pow = function(x, y) {};

/**
 * @return {number}
 * @nosideeffects
 * @see http://developer.mozilla.org/en/Core_JavaScript_1.5_Reference/Global_Objects/Math/random
 */
Math.random = function() {};

/**
 * @param {*} x
 * @return {number}
 * @nosideeffects
 * @see http://developer.mozilla.org/en/Core_JavaScript_1.5_Reference/Global_Objects/Math/round
 */
Math.round = function(x) {};

/**
 * @param {*} x
 * @return {number}
 * @nosideeffects
 * @see http://developer.mozilla.org/en/Core_JavaScript_1.5_Reference/Global_Objects/Math/sin
 */
Math.sin = function(x) {};

/**
 * @param {*} x
 * @return {number}
 * @nosideeffects
 * @see http://developer.mozilla.org/en/Core_JavaScript_1.5_Reference/Global_Objects/Math/sqrt
 */
Math.sqrt = function(x) {};

/**
 * @param {*} x
 * @return {number}
 * @nosideeffects
 * @see http://developer.mozilla.org/en/Core_JavaScript_1.5_Reference/Global_Objects/Math/tan
 */
Math.tan = function(x) {};

/**
 * @return {string}
 * @nosideeffects
 * @see http://developer.mozilla.org/en/Core_JavaScript_1.5_Reference/Global_Objects/Math/toSource
 */
Math.toSource = function() {};

// Properties:

/**
 * @type {number}
 * @see http://developer.mozilla.org/en/Core_JavaScript_1.5_Reference/Global_Objects/Math/E
 */
Math.E;

/**
 * @type {number}
 * @see http://developer.mozilla.org/en/Core_JavaScript_1.5_Reference/Global_Objects/Math/LN2
 */
Math.LN2;

/**
 * @type {number}
 * @see http://developer.mozilla.org/en/Core_JavaScript_1.5_Reference/Global_Objects/Math/LN10
 */
Math.LN10;

/**
 * @type {number}
 * @see http://developer.mozilla.org/en/Core_JavaScript_1.5_Reference/Global_Objects/Math/LOG2E
 */
Math.LOG2E;

/**
 * @type {number}
 * @see http://developer.mozilla.org/en/Core_JavaScript_1.5_Reference/Global_Objects/Math/LOG10E
 */
Math.LOG10E;

/**
 * @type {number}
 * @see http://developer.mozilla.org/en/Core_JavaScript_1.5_Reference/Global_Objects/Math/PI
 */
Math.PI;

/**
 * @type {number}
 * @see http://developer.mozilla.org/en/Core_JavaScript_1.5_Reference/Global_Objects/Math/SQRT1_2
 */
Math.SQRT1_2;

/**
 * @type {number}
 * @see http://developer.mozilla.org/en/Core_JavaScript_1.5_Reference/Global_Objects/Math/SQRT2
 */
Math.SQRT2;


/**
 * @param {?=} opt_yr_num
 * @param {?=} opt_mo_num
 * @param {?=} opt_day_num
 * @param {?=} opt_hr_num
 * @param {?=} opt_min_num
 * @param {?=} opt_sec_num
 * @param {?=} opt_ms_num
 * @constructor
 * @return {string}
 * @nosideeffects
 * @see http://developer.mozilla.org/en/Core_JavaScript_1.5_Reference/Global_Objects/Date
 */
function Date(opt_yr_num, opt_mo_num, opt_day_num, opt_hr_num, opt_min_num,
    opt_sec_num, opt_ms_num) {}

/**
 * @return {number}
 * @nosideeffects
 * @see http://developer.mozilla.org/en/Core_JavaScript_1.5_Reference/Global_Objects/Date/now
 */
Date.now = function() {};

/**
 * Parses a string representation of a date, and returns the number
 * of milliseconds since January 1, 1970, 00:00:00, local time.
 * @param {*} date
 * @return {number}
 * @nosideeffects
 * @see http://developer.mozilla.org/en/Core_JavaScript_1.5_Reference/Global_Objects/Date/parse
 */
Date.parse = function(date) {};

/**
 * @param {number} year
 * @param {number} month
 * @param {number=} opt_date
 * @param {number=} opt_hours
 * @param {number=} opt_minute
 * @param {number=} opt_second
 * @param {number=} opt_ms
 * @return {number}
 * @nosideeffects
 * @see http://developer.mozilla.org/en/Core_JavaScript_1.5_Reference/Global_Objects/Date/UTC
 */
Date.UTC = function(year, month,
                    opt_date, opt_hours, opt_minute, opt_second, opt_ms) {};

/**
 * @return {number}
 * @nosideeffects
 * @see http://developer.mozilla.org/en/Core_JavaScript_1.5_Reference/Global_Objects/Date/getDate
 */
Date.prototype.getDate = function() {};

/**
 * @return {number}
 * @nosideeffects
 * @see http://developer.mozilla.org/en/Core_JavaScript_1.5_Reference/Global_Objects/Date/getDay
 */
Date.prototype.getDay = function() {};

/**
 * @return {number}
 * @nosideeffects
 * @see http://developer.mozilla.org/en/Core_JavaScript_1.5_Reference/Global_Objects/Date/getMonth
 */
Date.prototype.getMonth = function() {};

/**
 * @return {number}
 * @nosideeffects
 * @see http://developer.mozilla.org/en/Core_JavaScript_1.5_Reference/Global_Objects/Date/getFullYear
 */
Date.prototype.getFullYear = function() {};

/**
 * @return {number}
 * @nosideeffects
 * @see http://developer.mozilla.org/en/Core_JavaScript_1.5_Reference/Global_Objects/Date/getYear
 */
Date.prototype.getYear = function() {};

/**
 * @return {number}
 * @nosideeffects
 * @see http://developer.mozilla.org/en/Core_JavaScript_1.5_Reference/Global_Objects/Date/getHours
 */
Date.prototype.getHours = function() {};

/**
 * @return {number}
 * @nosideeffects
 * @see http://developer.mozilla.org/en/Core_JavaScript_1.5_Reference/Global_Objects/Date/getMinutes
 */
Date.prototype.getMinutes = function() {};

/**
 * @return {number}
 * @nosideeffects
 * @see http://developer.mozilla.org/en/Core_JavaScript_1.5_Reference/Global_Objects/Date/getSeconds
 */
Date.prototype.getSeconds = function() {};

/**
 * @return {number}
 * @nosideeffects
 * @see http://developer.mozilla.org/en/Core_JavaScript_1.5_Reference/Global_Objects/Date/getMilliseconds
 */
Date.prototype.getMilliseconds = function() {};

/**
 * @return {number}
 * @nosideeffects
 * @see http://developer.mozilla.org/en/Core_JavaScript_1.5_Reference/Global_Objects/Date/getTime
 */
Date.prototype.getTime = function() {};

/**
 * @return {number}
 * @nosideeffects
 * @see http://developer.mozilla.org/en/Core_JavaScript_1.5_Reference/Global_Objects/Date/getTimezoneOffset
 */
Date.prototype.getTimezoneOffset = function() {};

/**
 * @return {number}
 * @nosideeffects
 * @see http://developer.mozilla.org/en/Core_JavaScript_1.5_Reference/Global_Objects/Date/getUTCDate
 */
Date.prototype.getUTCDate = function() {};

/**
 * @return {number}
 * @nosideeffects
 * @see http://developer.mozilla.org/en/Core_JavaScript_1.5_Reference/Global_Objects/Date/getUTCDay
 */
Date.prototype.getUTCDay = function() {};

/**
 * @return {number}
 * @nosideeffects
 * @see http://developer.mozilla.org/en/Core_JavaScript_1.5_Reference/Global_Objects/Date/getUTCMonth
 */
Date.prototype.getUTCMonth = function() {};

/**
 * @return {number}
 * @nosideeffects
 * @see http://developer.mozilla.org/en/Core_JavaScript_1.5_Reference/Global_Objects/Date/getUTCFullYear
 */
Date.prototype.getUTCFullYear = function() {};

/**
 * @return {number}
 * @nosideeffects
 * @see http://developer.mozilla.org/en/Core_JavaScript_1.5_Reference/Global_Objects/Date/getUTCHours
 */
Date.prototype.getUTCHours = function() {};

/**
 * @return {number}
 * @nosideeffects
 * @see http://developer.mozilla.org/en/Core_JavaScript_1.5_Reference/Global_Objects/Date/getUTCMinutes
 */
Date.prototype.getUTCMinutes = function() {};

/**
 * @return {number}
 * @nosideeffects
 * @see http://developer.mozilla.org/en/Core_JavaScript_1.5_Reference/Global_Objects/Date/getUTCSeconds
 */
Date.prototype.getUTCSeconds = function() {};

/**
 * @return {number}
 * @nosideeffects
 * @see http://developer.mozilla.org/en/Core_JavaScript_1.5_Reference/Global_Objects/Date/getUTCMilliseconds
 */
Date.prototype.getUTCMilliseconds = function() {};

/**
 * Sets the day of the month for a specified date according to local time.
 *
 * @param {number} dayValue
 * @modifies {this}
 * @see http://developer.mozilla.org/en/Core_JavaScript_1.5_Reference/Global_Objects/Date/setDate
 */
Date.prototype.setDate = function(dayValue) {};

/**
 * Set the month for a specified date according to local time.
 *
 * @param {number} monthValue
 * @param {number=} opt_dayValue
 * @modifies {this}
 * @see http://developer.mozilla.org/en/Core_JavaScript_1.5_Reference/Global_Objects/Date/setMonth
 */
Date.prototype.setMonth = function(monthValue, opt_dayValue) {};

/**
 * Sets the full year for a specified date according to local time.
 *
 * @param {number} yearValue
 * @param {number=} opt_monthValue
 * @param {number=} opt_dayValue
 * @modifies {this}
 * @see http://developer.mozilla.org/en/Core_JavaScript_1.5_Reference/Global_Objects/Date/setFullYear
 */
Date.prototype.setFullYear =
    function(yearValue, opt_monthValue, opt_dayValue) {};

/**
 * Sets the year for a specified date according to local time.
 *
 * @param {number} yearValue
 * @deprecated
 * @modifies {this}
 * @see http://developer.mozilla.org/en/Core_JavaScript_1.5_Reference/Global_Objects/Date/setYear
 */
Date.prototype.setYear = function(yearValue) {};

/**
 * Sets the hours for a specified date according to local time.
 *
 * @param {number} hoursValue
 * @param {number=} opt_minutesValue
 * @param {number=} opt_secondsValue
 * @param {number=} opt_msValue
 * @modifies {this}
 * @see http://developer.mozilla.org/en/Core_JavaScript_1.5_Reference/Global_Objects/Date/setHours
 */
Date.prototype.setHours = function(hoursValue, opt_minutesValue,
                                   opt_secondsValue, opt_msValue) {};

/**
 * Sets the minutes for a specified date according to local time.
 *
 * @param {number} minutesValue
 * @param {number=} opt_secondsValue
 * @param {number=} opt_msValue
 * @modifies {this}
 * @see http://developer.mozilla.org/en/Core_JavaScript_1.5_Reference/Global_Objects/Date/setMinutes
 */
Date.prototype.setMinutes =
    function(minutesValue, opt_secondsValue, opt_msValue) {};

/**
 * Sets the seconds for a specified date according to local time.
 *
 * @param {number} secondsValue
 * @param {number=} opt_msValue
 * @modifies {this}
 * @see http://developer.mozilla.org/en/Core_JavaScript_1.5_Reference/Global_Objects/Date/setSeconds
 */
Date.prototype.setSeconds = function(secondsValue, opt_msValue) {};

/**
 * Sets the milliseconds for a specified date according to local time.
 *
 * @param {number} millisecondsValue
 * @modifies {this}
 * @see http://developer.mozilla.org/en/Core_JavaScript_1.5_Reference/Global_Objects/Date/setMilliseconds
 */
Date.prototype.setMilliseconds = function(millisecondsValue) {};

/**
 * Sets the Date object to the time represented by a number of milliseconds
 * since January 1, 1970, 00:00:00 UTC.
 *
 * @param {number} timeValue
 * @modifies {this}
 * @see http://developer.mozilla.org/en/Core_JavaScript_1.5_Reference/Global_Objects/Date/setTime
 */
Date.prototype.setTime = function(timeValue) {};

/**
 * Sets the day of the month for a specified date according to universal time.
 *
 * @param {number} dayValue
 * @modifies {this}
 * @see http://developer.mozilla.org/en/Core_JavaScript_1.5_Reference/Global_Objects/Date/setUTCDate
 */
Date.prototype.setUTCDate = function(dayValue) {};

/**
 * Sets the month for a specified date according to universal time.
 *
 * @param {number} monthValue
 * @param {number=} opt_dayValue
 * @modifies {this}
 * @see http://developer.mozilla.org/en/Core_JavaScript_1.5_Reference/Global_Objects/Date/setUTCMonth
 */
Date.prototype.setUTCMonth = function(monthValue, opt_dayValue) {};

/**
 * Sets the full year for a specified date according to universal time.
 *
 * @param {number} yearValue
 * @param {number=} opt_monthValue
 * @param {number=} opt_dayValue
 * @modifies {this}
 * @see http://developer.mozilla.org/en/Core_JavaScript_1.5_Reference/Global_Objects/Date/setUTCFullYear
 */
Date.prototype.setUTCFullYear = function(yearValue, opt_monthValue,
                                         opt_dayValue) {};

/**
 * Sets the hour for a specified date according to universal time.
 *
 * @param {number} hoursValue
 * @param {number=} opt_minutesValue
 * @param {number=} opt_secondsValue
 * @param {number=} opt_msValue
 * @modifies {this}
 * @see http://developer.mozilla.org/en/Core_JavaScript_1.5_Reference/Global_Objects/Date/setUTCHours
 */
Date.prototype.setUTCHours = function(hoursValue, opt_minutesValue,
                                      opt_secondsValue, opt_msValue) {};

/**
 * Sets the minutes for a specified date according to universal time.
 *
 * @param {number} minutesValue
 * @param {number=} opt_secondsValue
 * @param {number=} opt_msValue
 * @modifies {this}
 * @see http://developer.mozilla.org/en/Core_JavaScript_1.5_Reference/Global_Objects/Date/setUTCMinutes
 */
Date.prototype.setUTCMinutes = function(minutesValue, opt_secondsValue,
                                        opt_msValue) {};


/**
 * Sets the seconds for a specified date according to universal time.
 *
 * @param {number} secondsValue
 * @param {number=} opt_msValue
 * @modifies {this}
 * @see http://developer.mozilla.org/en/Core_JavaScript_1.5_Reference/Global_Objects/Date/setUTCSeconds
 */
Date.prototype.setUTCSeconds = function(secondsValue, opt_msValue) {};

/**
 * Sets the milliseconds for a specified date according to universal time.
 *
 * @param {number} millisecondsValue
 * @modifies {this}
 * @see http://developer.mozilla.org/en/Core_JavaScript_1.5_Reference/Global_Objects/Date/setUTCMilliseconds
 */
Date.prototype.setUTCMilliseconds = function(millisecondsValue) {};

/**
 * @return {string}
 * @nosideeffects
 * @see http://developer.mozilla.org/en/Core_JavaScript_1.5_Reference/Global_Objects/Date/toSource
 * @override
 */
Date.prototype.toSource = function() {};

/**
 * @return {string}
 * @nosideeffects
 * @see http://developer.mozilla.org/en/JavaScript/Reference/Global_Objects/Date/toDateString
 */
Date.prototype.toDateString = function() {};

/**
 * @return {string}
 * @nosideeffects
 * @see http://developer.mozilla.org/en/Core_JavaScript_1.5_Reference/Global_Objects/Date/toGMTString
 */
Date.prototype.toGMTString = function() {};

/**
 * @return {string}
 * @nosideeffects
 * @see http://developer.mozilla.org/en/Core_JavaScript_1.5_Reference/Global_Objects/Date/toTimeString
 */
Date.prototype.toTimeString = function() {};

/**
 * @return {string}
 * @nosideeffects
 * @see http://developer.mozilla.org/en/Core_JavaScript_1.5_Reference/Global_Objects/Date/toUTCString
 */
Date.prototype.toUTCString = function() {};

/**
 * @param {(string|Array.<string>)=} opt_locales
 * @param {Object=} opt_options
 * @return {string}
 * @nosideeffects
 * @see http://developer.mozilla.org/en/Core_JavaScript_1.5_Reference/Global_Objects/Date/toLocaleDateString
 */
Date.prototype.toLocaleDateString = function(opt_locales, opt_options) {};

/**
 * @param {string} formatString
 * @return {string}
 * @nosideeffects
 * @see http://developer.mozilla.org/en/Core_JavaScript_1.5_Reference/Global_Objects/Date/toLocaleFormat
 */
Date.prototype.toLocaleFormat = function(formatString) {};

/**
 * @param {string|Array.<string>=} opt_locales
 * @param {Object=} opt_options
 * @return {string}
 * @nosideeffects
 * @see http://developer.mozilla.org/en/Core_JavaScript_1.5_Reference/Global_Objects/Date/toLocaleString
 * @see http://www.ecma-international.org/ecma-402/1.0/#sec-13.3.1
 * @override
 */
Date.prototype.toLocaleString = function(opt_locales, opt_options) {};

/**
 * @param {(string|Array.<string>)=} opt_locales
 * @param {Object=} opt_options
 * @return {string}
 * @nosideeffects
 * @see http://developer.mozilla.org/en/Core_JavaScript_1.5_Reference/Global_Objects/Date/toLocaleTimeString
 */
Date.prototype.toLocaleTimeString = function(opt_locales, opt_options) {};

/**
 * @this {Date}
 * @return {string}
 * @nosideeffects
 * @override
 */
Date.prototype.toString = function() {};

/**
 * @return {number}
 * @nosideeffects
 * @see http://developer.mozilla.org/en/Core_JavaScript_1.5_Reference/Global_Objects/Date/valueOf
 */
Date.prototype.valueOf;

/**
 * @constructor
 * @param {*=} opt_str
 * @return {string}
 * @nosideeffects
 * @see http://developer.mozilla.org/en/Core_JavaScript_1.5_Reference/Global_Objects/String
 */
function String(opt_str) {}
// Functions:

/**
 * @param {...number} var_args
 * @return {string}
 * @nosideeffects
 * @see http://developer.mozilla.org/en/Core_JavaScript_1.5_Reference/Global_Objects/String/fromCharCode
 */
String.fromCharCode = function(var_args) {};

/**
 * @this {String|string}
 * @return {string}
 * @nosideeffects
 * @see http://developer.mozilla.org/en/Core_JavaScript_1.5_Reference/Global_Objects/String/anchor
 */
String.prototype.anchor = function() {};

/**
 * @this {String|string}
 * @return {string}
 * @nosideeffects
 * @see http://developer.mozilla.org/en/Core_JavaScript_1.5_Reference/Global_Objects/String/big
 */
String.prototype.big = function() {};

/**
 * @this {String|string}
 * @return {string}
 * @nosideeffects
 * @see http://developer.mozilla.org/en/Core_JavaScript_1.5_Reference/Global_Objects/String/blink
 */
String.prototype.blink = function() {};

/**
 * @this {String|string}
 * @return {string}
 * @nosideeffects
 * @see http://developer.mozilla.org/en/Core_JavaScript_1.5_Reference/Global_Objects/String/bold
 */
String.prototype.bold = function() {};

/**
 * Returns the specified character from a string.
 *
 * @this {String|string}
 * @param {number} index
 * @return {string}
 * @nosideeffects
 * @see http://developer.mozilla.org/en/Core_JavaScript_1.5_Reference/Global_Objects/String/charAt
 */
String.prototype.charAt = function(index) {};

/**
 * Returns a number indicating the Unicode value of the character at the given
 * index.
 *
 * @this {String|string}
 * @param {number=} opt_index
 * @return {number}
 * @nosideeffects
 * @see http://developer.mozilla.org/en/Core_JavaScript_1.5_Reference/Global_Objects/String/charCodeAt
 */
String.prototype.charCodeAt = function(opt_index) {};

/**
 * Combines the text of two or more strings and returns a new string.
 *
 * @this {String|string}
 * @param {...*} var_args
 * @return {string}
 * @nosideeffects
 * @see http://developer.mozilla.org/en/Core_JavaScript_1.5_Reference/Global_Objects/String/concat
 */
String.prototype.concat = function(var_args) {};

/**
 * @this {String|string}
 * @return {string}
 * @nosideeffects
 * @see http://developer.mozilla.org/en/Core_JavaScript_1.5_Reference/Global_Objects/String/fixed
 */
String.prototype.fixed = function() {};

/**
 * @this {String|string}
 * @param {string} color
 * @return {string}
 * @nosideeffects
 * @see http://developer.mozilla.org/en/Core_JavaScript_1.5_Reference/Global_Objects/String/fontcolor
 */
String.prototype.fontcolor = function(color) {};

/**
 * @this {String|string}
 * @param {number} size
 * @return {string}
 * @nosideeffects
 * @see http://developer.mozilla.org/en/Core_JavaScript_1.5_Reference/Global_Objects/String/fontsize
 */
String.prototype.fontsize = function(size) {};

/**
 * Returns the index within the calling String object of the first occurrence
 * of the specified value, starting the search at fromIndex, returns -1 if the
 * value is not found.
 *
 * @this {String|string}
 * @param {string|null} searchValue
 * @param {(number|null)=} opt_fromIndex
 * @return {number}
 * @nosideeffects
 * @see http://developer.mozilla.org/en/Core_JavaScript_1.5_Reference/Global_Objects/String/indexOf
 */
String.prototype.indexOf = function(searchValue, opt_fromIndex) {};

/**
 * @this {String|string}
 * @return {string}
 * @nosideeffects
 * @see http://developer.mozilla.org/en/Core_JavaScript_1.5_Reference/Global_Objects/String/italics
 */
String.prototype.italics = function() {};

/**
 * Returns the index within the calling String object of the last occurrence of
 * the specified value, or -1 if not found. The calling string is searched
 * backward, starting at fromIndex.
 *
 * @this {String|string}
 * @param {string|null} searchValue
 * @param {(number|null)=} opt_fromIndex
 * @return {number}
 * @nosideeffects
 * @see http://developer.mozilla.org/en/Core_JavaScript_1.5_Reference/Global_Objects/String/lastIndexOf
 */
String.prototype.lastIndexOf = function(searchValue, opt_fromIndex) {};

/**
 * @this {String|string}
 * @param {string} hrefAttribute
 * @return {string}
 * @nosideeffects
 * @see http://developer.mozilla.org/en/Core_JavaScript_1.5_Reference/Global_Objects/String/link
 */
String.prototype.link = function(hrefAttribute) {};

/**
 * Returns a number indicating whether a reference string comes before or after
 * or is the same as the given string in sort order.
 *
 * @this {*}
 * @param {?string} compareString
 * @param {string|Array.<string>=} locales
 * @param {Object=} options
 * @return {number}
 * @nosideeffects
 * @see http://developer.mozilla.org/En/Core_JavaScript_1.5_Reference/Objects/String/localeCompare
 * @see http://www.ecma-international.org/ecma-402/1.0/#sec-13.1.1
 */
String.prototype.localeCompare = function(compareString, locales, options) {};

/**
 * Used to retrieve the matches when matching a string against a regular
 * expression.
 *
 * @this {String|string}
 * @param {*} regexp
 * @return {Array.<string>} This should really return an Array with a few
 *     special properties, but we do not have a good way to model this in
 *     our type system. Also see Regexp.prototype.exec.
 * @see http://developer.mozilla.org/en/Core_JavaScript_1.5_Reference/Global_Objects/String/match
 */
String.prototype.match = function(regexp) {};

/**
 * @this {String|string}
 * @return {string}
 * @nosideeffects
 * @see http://developer.mozilla.org/en/Core_JavaScript_1.5_Reference/Global_Objects/String/quote
 */
String.prototype.quote = function() {};

/**
 * Finds a match between a regular expression and a string, and replaces the
 * matched substring with a new substring.
 *
 * This may have side-effects if the replacement function has side-effects.
 *
 * @this {String|string}
 * @param {RegExp|string} regex
 * @param {string|Function} str
 * @param {string=} opt_flags
 * @return {string}
 * @see http://developer.mozilla.org/en/Core_JavaScript_1.5_Reference/Global_Objects/String/replace
 */
String.prototype.replace = function(regex, str, opt_flags) {};

/**
 * Executes the search for a match between a regular expression and this String
 * object.
 *
 * @this {String|string}
 * @param {RegExp|string} regexp
 * @return {number}
 * @see http://developer.mozilla.org/en/Core_JavaScript_1.5_Reference/Global_Objects/String/search
 */
String.prototype.search = function(regexp) {};

/**
 * @this {String|string}
 * @param {number} begin
 * @param {number=} opt_end
 * @return {string}
 * @nosideeffects
 * @see http://developer.mozilla.org/en/Core_JavaScript_1.5_Reference/Global_Objects/String/slice
 */
String.prototype.slice = function(begin, opt_end) {};

/**
 * @this {String|string}
 * @return {string}
 * @nosideeffects
 * @see http://developer.mozilla.org/en/Core_JavaScript_1.5_Reference/Global_Objects/String/small
 */
String.prototype.small = function() {};

/**
 * @this {String|string}
 * @param {*=} opt_separator
 * @param {number=} opt_limit
 * @return {!Array.<string>}
 * @nosideeffects
 * @see http://developer.mozilla.org/en/Core_JavaScript_1.5_Reference/Global_Objects/String/split
 */
String.prototype.split = function(opt_separator, opt_limit) {};

/**
 * @return {string}
 * @nosideeffects
 * @see http://developer.mozilla.org/en/Core_JavaScript_1.5_Reference/Global_Objects/String/strike
 */
String.prototype.strike = function() {};

/**
 * @this {String|string}
 * @return {string}
 * @nosideeffects
 * @see http://developer.mozilla.org/en/Core_JavaScript_1.5_Reference/Global_Objects/String/sub
 */
String.prototype.sub = function() {};

/**
 * @this {String|string}
 * @param {number} start
 * @param {number=} opt_length
 * @return {string} The specified substring.
 * @nosideeffects
 * @see http://developer.mozilla.org/en/Core_JavaScript_1.5_Reference/Global_Objects/String/substr
 */
String.prototype.substr = function(start, opt_length) {};

/**
 * @this {String|string}
 * @param {number} start
 * @param {number=} opt_end
 * @return {string} The specified substring.
 * @nosideeffects
 * @see http://developer.mozilla.org/en/Core_JavaScript_1.5_Reference/Global_Objects/String/substring
 */
String.prototype.substring = function(start, opt_end) {};

/**
 * @this {String|string}
 * @return {string}
 * @nosideeffects
 * @see http://developer.mozilla.org/en/Core_JavaScript_1.5_Reference/Global_Objects/String/sup
 */
String.prototype.sup = function() {};

/**
 * @this {String|string}
 * @return {string}
 * @nosideeffects
 * @see http://developer.mozilla.org/en/Core_JavaScript_1.5_Reference/Global_Objects/String/toLocaleUpperCase
 */
String.prototype.toLocaleUpperCase = function() {};

/**
 * @this {String|string}
 * @return {string}
 * @nosideeffects
 * @see http://developer.mozilla.org/en/Core_JavaScript_1.5_Reference/Global_Objects/String/toLocaleLowerCase
 */
String.prototype.toLocaleLowerCase = function() {};

/**
 * @this {String|string}
 * @return {string}
 * @nosideeffects
 * @see http://developer.mozilla.org/en/Core_JavaScript_1.5_Reference/Global_Objects/String/toLowerCase
 */
String.prototype.toLowerCase = function() {};

/**
 * @this {String|string}
 * @return {string}
 * @nosideeffects
 * @see http://developer.mozilla.org/en/Core_JavaScript_1.5_Reference/Global_Objects/String/toUpperCase
 */
String.prototype.toUpperCase = function() {};

/**
 * @return {string}
 * @nosideeffects
 * @see http://developer.mozilla.org/en/Core_JavaScript_1.5_Reference/Global_Objects/String/toSource
 * @override
 */
String.prototype.toSource = function() {};

/**
 * @this {string|String}
 * @return {string}
 * @nosideeffects
 * @override
 */
String.prototype.toString = function() {};

/**
 * @return {string}
 * @nosideeffects
 * @see http://developer.mozilla.org/en/Core_JavaScript_1.5_Reference/Global_Objects/String/valueOf
 */
String.prototype.valueOf;

/**
 * @type {number}
 * @see http://developer.mozilla.org/en/Core_JavaScript_1.5_Reference/Global_Objects/String/length
 */
String.prototype.length;

/**
 * @constructor
 * @param {*=} opt_pattern
 * @param {*=} opt_flags
 * @return {!RegExp}
 * @nosideeffects
 * @see http://developer.mozilla.org/en/Core_JavaScript_1.5_Reference/Global_Objects/RegExp
 */
function RegExp(opt_pattern, opt_flags) {}

/**
 * @param {*} pattern
 * @param {*=} opt_flags
 * @return {void}
 * @modifies {this}
 * @deprecated
 * @see http://msdn.microsoft.com/en-us/library/x9cswe0z(v=VS.85).aspx
 * @see http://developer.mozilla.org/en/Core_JavaScript_1.5_Reference/Global_Objects/RegExp/compile
 */
RegExp.prototype.compile = function(pattern, opt_flags) {};

/**
 * @param {*} str The string to search.
 * @return {Array.<string>} This should really return an Array with a few
 *     special properties, but we do not have a good way to model this in
 *     our type system. Also see String.prototype.match.
 * @see http://msdn.microsoft.com/en-us/library/z908hy33(VS.85).aspx
 * @see http://developer.mozilla.org/en/Core_JavaScript_1.5_Reference/Global_Objects/RegExp/exec
 */
RegExp.prototype.exec = function(str) {};

/**
 * @param {*} str The string to search.
 * @return {boolean} Whether the string was matched.
 * @see http://developer.mozilla.org/en/Core_JavaScript_1.5_Reference/Global_Objects/RegExp/test
 */
RegExp.prototype.test = function(str) {};

/**
 * @this {RegExp}
 * @return {string}
 * @nosideeffects
 * @override
 */
RegExp.prototype.toString = function() {};

// Constructor properties:

/**
 * The string against which the last regexp was matched.
 * @type {string}
 * @see http://www.devguru.com/Technologies/Ecmascript/Quickref/regexp_input.html
 */
RegExp.input;

/**
 * The last matched characters.
 * @type {string}
 * @see http://www.devguru.com/Technologies/Ecmascript/Quickref/regexp_lastMatch.html
 */
RegExp.lastMatch;

/**
 * The last matched parenthesized substring, if any.
 * @type {string}
 * @see http://www.devguru.com/Technologies/Ecmascript/Quickref/regexp_lastParen.html
 */
RegExp.lastParen;

/**
 * The substring of the input up to the characters most recently matched.
 * @type {string}
 * @see http://www.devguru.com/Technologies/Ecmascript/Quickref/regexp_leftContext.html
 */
RegExp.leftContext;

/**
 * The substring of the input after the characters most recently matched.
 * @type {string}
 * @see http://www.devguru.com/Technologies/Ecmascript/Quickref/regexp_rightContext.html
 */
RegExp.rightContext;

/**
 * @type {string}
 * @see http://developer.mozilla.org/en/Core_JavaScript_1.5_Reference/Global_Objects/RegExp
 */
RegExp.$1;
/**
 * @type {string}
 * @see http://developer.mozilla.org/en/Core_JavaScript_1.5_Reference/Global_Objects/RegExp
 */
RegExp.$2;
/**
 * @type {string}
 * @see http://developer.mozilla.org/en/Core_JavaScript_1.5_Reference/Global_Objects/RegExp
 */
RegExp.$3;
/**
 * @type {string}
 * @see http://developer.mozilla.org/en/Core_JavaScript_1.5_Reference/Global_Objects/RegExp
 */
RegExp.$4;
/**
 * @type {string}
 * @see http://developer.mozilla.org/en/Core_JavaScript_1.5_Reference/Global_Objects/RegExp
 */
RegExp.$5;
/**
 * @type {string}
 * @see http://developer.mozilla.org/en/Core_JavaScript_1.5_Reference/Global_Objects/RegExp
 */
RegExp.$6;
/**
 * @type {string}
 * @see http://developer.mozilla.org/en/Core_JavaScript_1.5_Reference/Global_Objects/RegExp
 */
RegExp.$7;
/**
 * @type {string}
 * @see http://developer.mozilla.org/en/Core_JavaScript_1.5_Reference/Global_Objects/RegExp
 */
RegExp.$8;
/**
 * @type {string}
 * @see http://developer.mozilla.org/en/Core_JavaScript_1.5_Reference/Global_Objects/RegExp
 */
RegExp.$9;

// Prototype properties:

/**
 * Whether to test the regular expression against all possible matches
 * in a string, or only against the first.
 * @type {boolean}
 * @see http://developer.mozilla.org/en/Core_JavaScript_1.5_Reference/Global_Objects/RegExp/global
 */
RegExp.prototype.global;

/**
 * Whether to ignore case while attempting a match in a string.
 * @type {boolean}
 * @see http://developer.mozilla.org/en/Core_JavaScript_1.5_Reference/Global_Objects/RegExp/ignoreCase
 */
RegExp.prototype.ignoreCase;

/**
 * The index at which to start the next match.
 * @type {number}
 * @see http://developer.mozilla.org/en/Core_JavaScript_1.5_Reference/Global_Objects/RegExp/lastIndex
 */
RegExp.prototype.lastIndex;

/**
 * Whether or not to search in strings across multiple lines.
 * @type {boolean}
 * @see http://developer.mozilla.org/en/Core_JavaScript_1.5_Reference/Global_Objects/RegExp/multiline
 */
RegExp.prototype.multiline;

/**
 * The text of the pattern.
 * @type {string}
 * @see http://developer.mozilla.org/en/Core_JavaScript_1.5_Reference/Global_Objects/RegExp/source
 */
RegExp.prototype.source;


/**
 * @constructor
 * @param {*=} opt_message
 * @param {*=} opt_file
 * @param {*=} opt_line
 * @return {!Error}
 * @nosideeffects
 * @see http://developer.mozilla.org/en/Core_JavaScript_1.5_Reference/Global_Objects/Error
 */
function Error(opt_message, opt_file, opt_line) {}


/**
 * Chrome/v8 specific, altering the maximum depth of the stack trace
 * (10 by default).
 * @type {number}
 * @see http://code.google.com/p/v8/wiki/JavaScriptStackTraceApi
 */
Error.stackTraceLimit;


/**
 * Chrome/v8 specific, adds a stack trace to the error object. The optional
 * constructorOpt parameter allows you to pass in a function value. When
 * collecting the stack trace all frames above the topmost call to this
 * function, including that call, will be left out of the stack trace.
 * @param {Object} error The object to add the stack trace to.
 * @param {Function=} opt_constructor A function in the stack trace
 * @see http://code.google.com/p/v8/wiki/JavaScriptStackTraceApi
 */
Error.captureStackTrace = function(error, opt_constructor){};


/**
 * IE-only.
 * @type {string}
 * @see http://msdn.microsoft.com/en-us/library/2w6a45b5.aspx
 */
Error.prototype.description;


/**
 * Mozilla-only.
 * @type {number}
 * @see http://developer.mozilla.org/en/Core_JavaScript_1.5_Reference/Global_Objects/Error/lineNumber
 */
Error.prototype.lineNumber;

/**
 * Mozilla-only
 * @type {string}
 * @see https://developer.mozilla.org/en/Core_JavaScript_1.5_Reference/Global_Objects/Error/fileName
 */
Error.prototype.fileName;

/**
 * @type {string}
 * @see http://developer.mozilla.org/en/Core_JavaScript_1.5_Reference/Global_Objects/Error/name
 */
Error.prototype.name;

/**
 * @type {string}
 * @see http://developer.mozilla.org/en/Core_JavaScript_1.5_Reference/Global_Objects/Error/message
 */
Error.prototype.message;

/**
 * Doesn't seem to exist, but closure/debug.js references it.
 */
Error.prototype.sourceURL;

/** @type {string} */
Error.prototype.stack;


/**
 * @constructor
 * @extends {Error}
 * @param {*=} opt_message
 * @param {*=} opt_file
 * @param {*=} opt_line
 * @return {!EvalError}
 * @nosideeffects
 * @see http://developer.mozilla.org/en/Core_JavaScript_1.5_Reference/Global_Objects/EvalError
 */
function EvalError(opt_message, opt_file, opt_line) {}

/**
 * @constructor
 * @extends {Error}
 * @param {*=} opt_message
 * @param {*=} opt_file
 * @param {*=} opt_line
 * @return {!RangeError}
 * @nosideeffects
 * @see http://developer.mozilla.org/en/Core_JavaScript_1.5_Reference/Global_Objects/RangeError
 */
function RangeError(opt_message, opt_file, opt_line) {}

/**
 * @constructor
 * @extends {Error}
 * @param {*=} opt_message
 * @param {*=} opt_file
 * @param {*=} opt_line
 * @return {!ReferenceError}
 * @nosideeffects
 * @see http://developer.mozilla.org/en/Core_JavaScript_1.5_Reference/Global_Objects/ReferenceError
 */
function ReferenceError(opt_message, opt_file, opt_line) {}

/**
 * @constructor
 * @extends {Error}
 * @param {*=} opt_message
 * @param {*=} opt_file
 * @param {*=} opt_line
 * @return {!SyntaxError}
 * @nosideeffects
 * @see http://developer.mozilla.org/en/Core_JavaScript_1.5_Reference/Global_Objects/SyntaxError
 */
function SyntaxError(opt_message, opt_file, opt_line) {}

/**
 * @constructor
 * @extends {Error}
 * @param {*=} opt_message
 * @param {*=} opt_file
 * @param {*=} opt_line
 * @return {!TypeError}
 * @nosideeffects
 * @see http://developer.mozilla.org/en/Core_JavaScript_1.5_Reference/Global_Objects/TypeError
 */
function TypeError(opt_message, opt_file, opt_line) {}

/**
 * @constructor
 * @extends {Error}
 * @param {*=} opt_message
 * @param {*=} opt_file
 * @param {*=} opt_line
 * @return {!URIError}
 * @nosideeffects
 * @see http://developer.mozilla.org/en/Core_JavaScript_1.5_Reference/Global_Objects/URIError
 */
function URIError(opt_message, opt_file, opt_line) {}


// JScript extensions.
// @see http://msdn.microsoft.com/en-us/library/894hfyb4(VS.80).aspx

/**
 * @param {string} progId
 * @param {string=} opt_location
 * @constructor
 * @see http://msdn.microsoft.com/en-us/library/7sw4ddf8.aspx
 */
function ActiveXObject(progId, opt_location) {}

/**
 * @return {string}
 * @nosideeffects
 * @see http://msdn.microsoft.com/en-us/library/9k34bww2(VS.80).aspx
 */
function ScriptEngine() {}

/**
 * @return {number}
 * @nosideeffects
 * @see http://msdn.microsoft.com/en-us/library/yf25ky07(VS.80).aspx
 */
function ScriptEngineMajorVersion() {}

/**
 * @return {number}
 * @nosideeffects
 * @see http://msdn.microsoft.com/en-us/library/wx3812cz(VS.80).aspx
 */
function ScriptEngineMinorVersion() {}

/**
 * @return {number}
 * @nosideeffects
 * @see http://msdn.microsoft.com/en-us/library/e98hsk2f(VS.80).aspx
 */
function ScriptEngineBuildVersion() {}
