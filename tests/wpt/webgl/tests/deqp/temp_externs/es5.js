/*
 * Copyright 2009 The Closure Compiler Authors
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
 * @fileoverview Definitions for ECMAScript 5.
 * @see http://www.ecma-international.org/publications/files/drafts/tc39-2009-025.pdf
 * @externs
 */


/**
 * @param {Object|undefined} selfObj Specifies the object to which |this| should
 *     point when the function is run. If the value is null or undefined, it
 *     will default to the global object.
 * @param {...*} var_args Additional arguments that are partially
 *     applied to fn.
 * @return {!Function} A partially-applied form of the Function on which
 *     bind() was invoked as a method.
 * @nosideeffects
 * @see https://developer.mozilla.org/en/JavaScript/Reference/Global_Objects/Function/bind
 */
Function.prototype.bind = function(selfObj, var_args) {};


/**
 * @this {String|string}
 * @return {string}
 * @nosideeffects
 * @see http://developer.mozilla.org/en/JavaScript/Reference/Global_Objects/String/Trim
 */
String.prototype.trim = function() {};


/**
 * @this {String|string}
 * @return {string}
 * @nosideeffects
 * @see http://developer.mozilla.org/en/JavaScript/Reference/Global_Objects/String/TrimLeft
 */
String.prototype.trimLeft = function() {};


/**
 * @this {String|string}
 * @return {string}
 * @nosideeffects
 * @see http://developer.mozilla.org/en/JavaScript/Reference/Global_Objects/String/TrimRight
 */
String.prototype.trimRight = function() {};


/**
 * A object property descriptor used by Object.create, Object.defineProperty,
 * Object.defineProperties, Object.getOwnPropertyDescriptor.
 *
 * Note: not a real constructor.
 * @constructor
 */
var ObjectPropertyDescriptor = function(){};

/** @type {*} */
ObjectPropertyDescriptor.prototype.value;

/** @type {(function():?)||undefined} */
ObjectPropertyDescriptor.prototype.get;

/** @type {(function(?):void)||undefined} */
ObjectPropertyDescriptor.prototype.set;

/** @type {boolean|undefined} */
ObjectPropertyDescriptor.prototype.writable;

/** @type {boolean|undefined} */
ObjectPropertyDescriptor.prototype.enumerable;

/** @type {boolean|undefined} */
ObjectPropertyDescriptor.prototype.configurable;


/**
 * @param {Object} proto
 * @param {Object=} opt_properties  A map of ObjectPropertyDescriptors.
 * @return {!Object}
 * @nosideeffects
 * @see https://developer.mozilla.org/en/JavaScript/Reference/Global_Objects/Object/create
 */
Object.create = function(proto, opt_properties) {};


/**
 * @param {!Object} obj
 * @param {string} prop
 * @param {!Object} descriptor A ObjectPropertyDescriptor.
 * @return {!Object}
 * @see https://developer.mozilla.org/en/JavaScript/Reference/Global_Objects/Object/defineProperty
 */
Object.defineProperty = function(obj, prop, descriptor) {};


/**
 * @param {!Object} obj
 * @param {!Object} props A map of ObjectPropertyDescriptors.
 * @return {!Object}
 * @see https://developer.mozilla.org/en/JavaScript/Reference/Global_Objects/Object/defineProperties
 */
Object.defineProperties = function(obj, props) {};


/**
 * @param {!Object} obj
 * @param {string} prop
 * @return {!ObjectPropertyDescriptor|undefined}
 * @nosideeffects
 * @see https://developer.mozilla.org/en/JavaScript/Reference/Global_Objects/Object/getOwnPropertyDescriptor
 */
Object.getOwnPropertyDescriptor = function(obj, prop) {};


/**
 * @param {!Object} obj
 * @return {!Array.<string>}
 * @nosideeffects
 * @see https://developer.mozilla.org/en/JavaScript/Reference/Global_Objects/Object/keys
 */
Object.keys = function(obj) {};


/**
 * @param {!Object} obj
 * @return {!Array.<string>}
 * @nosideeffects
 * @see https://developer.mozilla.org/en/JavaScript/Reference/Global_Objects/Object/getOwnPropertyNames
 */
Object.getOwnPropertyNames = function(obj) {};


/**
 * @param {!Object} obj
 * @return {Object}
 * @nosideeffects
 * @see https://developer.mozilla.org/en/JavaScript/Reference/Global_Objects/Object/GetPrototypeOf
 */
Object.getPrototypeOf = function(obj) {};


/**
 * @param {T} obj
 * @return {T}
 * @template T
 * @see https://developer.mozilla.org/en/JavaScript/Reference/Global_Objects/Object/preventExtensions
 */
Object.preventExtensions = function(obj) {};


/**
 * @param {T} obj
 * @return {T}
 * @template T
 * @see https://developer.mozilla.org/en/JavaScript/Reference/Global_Objects/Object/seal
 */
Object.seal = function(obj) {};


/**
 * @param {T} obj
 * @return {T}
 * @template T
 * @see https://developer.mozilla.org/en/JavaScript/Reference/Global_Objects/Object/freeze
 */
Object.freeze = function(obj) {};


/**
 * @param {!Object} obj
 * @return {boolean}
 * @nosideeffects
 * @see https://developer.mozilla.org/en/JavaScript/Reference/Global_Objects/Object/isExtensible
 */
Object.isExtensible = function(obj) {};


/**
 * @param {!Object} obj
 * @return {boolean}
 * @nosideeffects
 * @see https://developer.mozilla.org/en/JavaScript/Reference/Global_Objects/Object/isSealed
 */
Object.isSealed = function(obj) {};


/**
 * @param {!Object} obj
 * @return {boolean}
 * @nosideeffects
 * @see https://developer.mozilla.org/en/JavaScript/Reference/Global_Objects/Object/isFrozen
 */
Object.isFrozen = function(obj) {};


/**
 * As per ECMAScript 5, 15.12.3.
 * @param {string=} opt_key The JSON key for this object.
 * @return {*} The serializable representation of this object. Note that this
 *     need not be a string. See http://goo.gl/PEUvs.
 */
Object.prototype.toJSON = function(opt_key) {};


/**
 * @see https://developer.mozilla.org/en/JavaScript/Reference/Global_Objects/Date/toISOString
 * @return {string}
 */
Date.prototype.toISOString = function() {};


/**
 * @param {*=} opt_ignoredKey
 * @return {string}
 * @override
 */
Date.prototype.toJSON = function(opt_ignoredKey) {};


/**
 * A fake type to model the JSON object.
 * @constructor
 */
var JSONType = function() {};


/**
 * @param {string} jsonStr The string to parse.
 * @param {(function(string, *) : *)=} opt_reviver
 * @return {*} The JSON object.
 * @throws {Error}
 * @nosideeffects
 */
JSONType.prototype.parse = function(jsonStr, opt_reviver) {};


/**
 * @param {*} jsonObj Input object.
 * @param {(Array.<string>|(function(string, *) : *)|null)=} opt_replacer
 * @param {(number|string)=} opt_space
 * @return {string} JSON string which represents jsonObj.
 * @throws {Error}
 * @nosideeffects
 */
JSONType.prototype.stringify = function(jsonObj, opt_replacer, opt_space) {};


/**
 * @type {!JSONType}
 * @suppress {duplicate}
 */
var JSON;
