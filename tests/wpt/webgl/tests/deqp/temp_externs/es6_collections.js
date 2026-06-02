/*
 * Copyright 2014 The Closure Compiler Authors
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
 * @fileoverview Definitions for ECMAScript 6.
 * @see http://wiki.ecmascript.org/doku.php?id=harmony:specification_drafts
 * @externs
 */

// TODO(johnlenz): Use Tuples for the Map and Set iterators where appropriate.

/**
 * @constructor
 * @param {Iterable.<!Array.<KEY|VALUE>>|!Array.<!Array.<KEY|VALUE>>=} opt_iterable
 * @implements {Iterable.<!Array.<KEY|VALUE>>}
 * @template KEY, VALUE
 * @see https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Map
 */
function Map(opt_iterable) {}

/** @return {void} */
Map.prototype.clear;

/**
 * @param {KEY} key
 * @return {boolean}
 */
Map.prototype.delete;

/**
 * @return {!Iterator.<!Array.<KEY|VALUE>>}
 * @nosideeffects
 */
Map.prototype.entries;

/**
 * @param {function(this:THIS, VALUE, KEY, MAP):void} callback
 * @param {THIS} thisArg
 * @this {MAP}
 * @template MAP,THIS
 */
Map.prototype.forEach;

/**
 * @param {KEY} key
 * @return {VALUE|undefined}
 * @nosideeffects
 */
Map.prototype.get;

/**
 * @param {KEY} key
 * @return {boolean}
 * @nosideeffects
 */
Map.prototype.has;

/**
 * @return {!Iterator.<KEY>}
 */
Map.prototype.keys;

/**
 * @param {KEY} key
 * @param {VALUE} value
 * @return {THIS}
 * @this {THIS}
 * @template THIS
 */
Map.prototype.set;

/**
 * @type {number}
 * (readonly)
 */
Map.prototype.size;

/**
 * @return {!Iterator.<VALUE>}
 * @nosideeffects
 */
Map.prototype.values;

/**
 * @return {!Iterator.<!Array.<KEY|VALUE>>}
 */
Map.prototype[Symbol.iterator] = function() {};


/**
 * @constructor
 * @param {Iterable.<!Array.<KEY|VALUE>>|!Array.<!Array.<KEY|VALUE>>=} opt_iterable
 * @template KEY, VALUE
 * @see https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/WeakMap
 */
function WeakMap(opt_iterable) {}

/** @return {void} */
WeakMap.prototype.clear;

/**
 * @param {KEY} key
 * @return {boolean}
 */
WeakMap.prototype.delete;

/**
 * @param {KEY} key
 * @return {VALUE|undefined}
 * @nosideeffects
 */
WeakMap.prototype.get;

/**
 * @param {KEY} key
 * @return {boolean}
 * @nosideeffects
 */
WeakMap.prototype.has;

/**
 * @param {KEY} key
 * @param {VALUE} value
 * @return {THIS}
 * @this {THIS}
 * @template THIS
 */
WeakMap.prototype.set;



/**
 * @constructor
 * @param {Iterable.<VALUE>|Array.<VALUE>=} opt_iterable
 * @implements {Iterable.<VALUE>}
 * @template VALUE
 * @see https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Set
 */
function Set(opt_iterable) {}

/**
 * @param {VALUE} value
 * @return {THIS}
 * @this {THIS}
 * @template THIS
 */
Set.prototype.add;

/**
 * @return {void}
 */
Set.prototype.clear;

/**
 * @param {VALUE} value
 * @return {boolean}
 */
Set.prototype.delete;

/**
 * @return {!Iterator.<!Array.<VALUE>>} Where each array has two entries:
 *     [value, value]
 * @nosideeffects
 */
Set.prototype.entries;

/**
 * @param {function(VALUE, VALUE, SET)} callback
 * @param {THIS} thisArg
 * @this {SET}
 * @template SET,THIS
 */
Set.prototype.forEach;

/**
 * @param {VALUE} value
 * @return {boolean}
 * @nosideeffects
 */
Set.prototype.has;

/**
 * @type {number} (readonly)
 */
Set.prototype.size;

/**
 * @return {!Iterator.<VALUE>}
 * @nosideeffects
 */
Set.prototype.keys;

/**
 * @return {!Iterator.<VALUE>}
 * @nosideeffects
 */
Set.prototype.values;

/**
 * @return {!Iterator.<VALUE>}
 */
Set.prototype[Symbol.iterator] = function() {};



/**
 * @constructor
 * @param {Iterable.<VALUE>|Array.<VALUE>=} opt_iterable
 * @template VALUE
 * @see https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Set
 */
function WeakSet(opt_iterable) {}

/**
 * @param {VALUE} value
 * @return {THIS}
 * @this {THIS}
 * @template THIS
 */
WeakSet.prototype.add;

/**
 * @return {void}
 */
WeakSet.prototype.clear;

/**
 * @param {VALUE} value
 * @return {boolean}
 */
WeakSet.prototype.delete;

/**
 * @param {VALUE} value
 * @return {boolean}
 * @nosideeffects
 */
WeakSet.prototype.has;


