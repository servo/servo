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
 * @see https://www.khronos.org/registry/typedarray/specs/latest/
 * @externs
 */

// TODO(johnlenz): symbol should be a primitive type.
/** @typedef {?} */
var symbol;

/**
 * @param {string} description
 * @return {symbol}
 */
function Symbol(description) {}

/** @const {symbol} */
Symbol.iterator;


/**
 * @interface
 * @template VALUE
 */
function Iterable() {}

// TODO(johnlenz): remove this when the compiler understands "symbol" natively
/**
 * @return {Iterator.<VALUE>}
 * @suppress {externsValidation}
 */
Iterable.prototype[Symbol.iterator] = function() {};



// TODO(johnlenz): Iterator should be a templated record type.
/**
 * @interface
 * @template VALUE
 * @see https://developer.mozilla.org/en-US/docs/Web/JavaScript/Guide/The_Iterator_protocol
 */
function Iterator() {}

/**
 * @param {VALUE=} value
 * @return {{value:VALUE, done:boolean}}
 */
Iterator.prototype.next;


/**
 * @constructor
 * @see http://people.mozilla.org/~jorendorff/es6-draft.html#sec-generator-objects
 * @implements {Iterator<VALUE>}
 * @template VALUE
 */
var Generator = function() {};

/**
 * @param {?=} opt_value
 * @return {{value:VALUE, done:boolean}}
 * @override
 */
Generator.prototype.next = function(opt_value) {};

/**
 * @param {VALUE} value
 * @return {{value:VALUE, done:boolean}}
 */
Generator.prototype.return = function(value) {};

/**
 * @param {?} exception
 * @return {{value:VALUE, done:boolean}}
 */
Generator.prototype.throw = function(exception) {};


// TODO(johnlenz): Array should be Iterable.



/**
 * @param {number} value
 * @return {number}
 * @nosideeffects
 */
Math.log10 = function(value) {};

/**
 * @param {number} value
 * @return {number}
 * @nosideeffects
 */
Math.log2 = function(value) {};

/**
 * @param {number} value
 * @return {number}
 * @nosideeffects
 */
Math.log1p = function(value) {};

/**
 * @param {number} value
 * @return {number}
 * @nosideeffects
 */
Math.expm1 = function(value) {};

/**
 * @param {number} value
 * @return {number}
 * @nosideeffects
 */
Math.cosh = function(value) {};

/**
 * @param {number} value
 * @return {number}
 * @nosideeffects
 */
Math.sinh = function(value) {};

/**
 * @param {number} value
 * @return {number}
 * @nosideeffects
 */
Math.tanh = function(value) {};

/**
 * @param {number} value
 * @return {number}
 * @nosideeffects
 */
Math.acosh = function(value) {};

/**
 * @param {number} value
 * @return {number}
 * @nosideeffects
 */
Math.asinh = function(value) {};

/**
 * @param {number} value
 * @return {number}
 * @nosideeffects
 */
Math.atanh = function(value) {};

/**
 * @param {number} value
 * @return {number}
 * @nosideeffects
 */
Math.trunc = function(value) {};

/**
 * @param {number} value
 * @return {number}
 * @nosideeffects
 */
Math.sign = function(value) {};

/**
 * @param {number} value
 * @return {number}
 * @nosideeffects
 */
Math.cbrt = function(value) {};

/**
 * @param {number} value1
 * @param {...number} var_args
 * @return {number}
 * @nosideeffects
 * @see http://people.mozilla.org/~jorendorff/es6-draft.html#sec-math.hypot
 */
Math.hypot = function(value1, var_args) {};


/**
 * @param {*} a
 * @param {*} b
 * @return {boolean}
 * @see http://people.mozilla.org/~jorendorff/es6-draft.html#sec-object.is
 */
Object.is;


/**
 * Returns a language-sensitive string representation of this number.
 * @param {(string|!Array<string>)=} opt_locales
 * @param {Object=} opt_options
 * @return {string}
 * @nosideeffects
 * @see https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Number/toLocaleString
 * @see http://www.ecma-international.org/ecma-402/1.0/#sec-13.2.1
 * @override
 */
Number.prototype.toLocaleString = function(opt_locales, opt_options) {};


/**
 * @see http://dev.w3.org/html5/postmsg/
 * @interface
 */
function Transferable() {}

/**
 * @param {number} length The length in bytes
 * @constructor
 * @noalias
 * @throws {Error}
 * @nosideeffects
 * @implements {Transferable}
 */
function ArrayBuffer(length) {}

/** @type {number} */
ArrayBuffer.prototype.byteLength;

/**
 * @param {number} begin
 * @param {number=} opt_end
 * @return {!ArrayBuffer}
 * @nosideeffects
 */
ArrayBuffer.prototype.slice = function(begin, opt_end) {};


/**
 * @constructor
 * @noalias
 */
function ArrayBufferView() {}

/** @type {!ArrayBuffer} */
ArrayBufferView.prototype.buffer;

/** @type {number} */
ArrayBufferView.prototype.byteOffset;

/** @type {number} */
ArrayBufferView.prototype.byteLength;


/**
 * @param {number|ArrayBufferView|Array.<number>|ArrayBuffer} length or array
 *     or buffer
 * @param {number=} opt_byteOffset
 * @param {number=} opt_length
 * @extends {ArrayBufferView}
 * @constructor
 * @noalias
 * @throws {Error}
 * @modifies {arguments} If the user passes a backing array, then indexed
 *     accesses will modify the backing array. JSCompiler does not model
 *     this well. In other words, if you have:
 *     <code>
 *     var x = new ArrayBuffer(1);
 *     var y = new Int8Array(x);
 *     y[0] = 2;
 *     </code>
 *     JSCompiler will not recognize that the last assignment modifies x.
 *     We workaround this by marking all these arrays as @modifies {arguments},
 *     to introduce the possibility that x aliases y.
 */
function Int8Array(length, opt_byteOffset, opt_length) {}

/** @type {number} */
Int8Array.BYTES_PER_ELEMENT;

/** @type {number} */
Int8Array.prototype.BYTES_PER_ELEMENT;

/** @type {number} */
Int8Array.prototype.length;

/**
 * @param {ArrayBufferView|Array.<number>} array
 * @param {number=} opt_offset
 */
Int8Array.prototype.set = function(array, opt_offset) {};

/**
 * @param {number} begin
 * @param {number=} opt_end
 * @return {!Int8Array}
 * @nosideeffects
 */
Int8Array.prototype.subarray = function(begin, opt_end) {};


/**
 * @param {number|ArrayBufferView|Array.<number>|ArrayBuffer} length or array
 *     or buffer
 * @param {number=} opt_byteOffset
 * @param {number=} opt_length
 * @extends {ArrayBufferView}
 * @constructor
 * @noalias
 * @throws {Error}
 * @modifies {arguments}
 */
function Uint8Array(length, opt_byteOffset, opt_length) {}

/** @type {number} */
Uint8Array.BYTES_PER_ELEMENT;

/** @type {number} */
Uint8Array.prototype.BYTES_PER_ELEMENT;

/** @type {number} */
Uint8Array.prototype.length;

/**
 * @param {ArrayBufferView|Array.<number>} array
 * @param {number=} opt_offset
 */
Uint8Array.prototype.set = function(array, opt_offset) {};

/**
 * @param {number} begin
 * @param {number=} opt_end
 * @return {!Uint8Array}
 * @nosideeffects
 */
Uint8Array.prototype.subarray = function(begin, opt_end) {};


/**
 * @param {number|ArrayBufferView|Array.<number>|ArrayBuffer} length or array
 *     or buffer
 * @param {number=} opt_byteOffset
 * @param {number=} opt_length
 * @extends {ArrayBufferView}
 * @constructor
 * @noalias
 * @throws {Error}
 * @modifies {arguments}
 */
function Uint8ClampedArray(length, opt_byteOffset, opt_length) {}

/** @type {number} */
Uint8ClampedArray.BYTES_PER_ELEMENT;

/** @type {number} */
Uint8ClampedArray.prototype.BYTES_PER_ELEMENT;

/** @type {number} */
Uint8ClampedArray.prototype.length;

/**
 * @param {ArrayBufferView|Array.<number>} array
 * @param {number=} opt_offset
 */
Uint8ClampedArray.prototype.set = function(array, opt_offset) {};

/**
 * @param {number} begin
 * @param {number=} opt_end
 * @return {!Uint8ClampedArray}
 * @nosideeffects
 */
Uint8ClampedArray.prototype.subarray = function(begin, opt_end) {};


/**
 * @typedef {Uint8ClampedArray}
 * @deprecated CanvasPixelArray has been replaced by Uint8ClampedArray
 *     in the latest spec.
 * @see http://www.w3.org/TR/2dcontext/#imagedata
 */
var CanvasPixelArray;


/**
 * @param {number|ArrayBufferView|Array.<number>|ArrayBuffer} length or array
 *     or buffer
 * @param {number=} opt_byteOffset
 * @param {number=} opt_length
 * @extends {ArrayBufferView}
 * @constructor
 * @noalias
 * @throws {Error}
 * @modifies {arguments}
 */
function Int16Array(length, opt_byteOffset, opt_length) {}

/** @type {number} */
Int16Array.BYTES_PER_ELEMENT;

/** @type {number} */
Int16Array.prototype.BYTES_PER_ELEMENT;

/** @type {number} */
Int16Array.prototype.length;

/**
 * @param {ArrayBufferView|Array.<number>} array
 * @param {number=} opt_offset
 */
Int16Array.prototype.set = function(array, opt_offset) {};

/**
 * @param {number} begin
 * @param {number=} opt_end
 * @return {!Int16Array}
 * @nosideeffects
 */
Int16Array.prototype.subarray = function(begin, opt_end) {};


/**
 * @param {number|ArrayBufferView|Array.<number>|ArrayBuffer} length or array
 *     or buffer
 * @param {number=} opt_byteOffset
 * @param {number=} opt_length
 * @extends {ArrayBufferView}
 * @constructor
 * @noalias
 * @throws {Error}
 * @modifies {arguments}
 */
function Uint16Array(length, opt_byteOffset, opt_length) {}

/** @type {number} */
Uint16Array.BYTES_PER_ELEMENT;

/** @type {number} */
Uint16Array.prototype.BYTES_PER_ELEMENT;

/** @type {number} */
Uint16Array.prototype.length;

/**
 * @param {ArrayBufferView|Array.<number>} array
 * @param {number=} opt_offset
 */
Uint16Array.prototype.set = function(array, opt_offset) {};

/**
 * @param {number} begin
 * @param {number=} opt_end
 * @return {!Uint16Array}
 * @nosideeffects
 */
Uint16Array.prototype.subarray = function(begin, opt_end) {};


/**
 * @param {number|ArrayBufferView|Array.<number>|ArrayBuffer} length or array
 *     or buffer
 * @param {number=} opt_byteOffset
 * @param {number=} opt_length
 * @extends {ArrayBufferView}
 * @constructor
 * @noalias
 * @throws {Error}
 * @modifies {arguments}
 */
function Int32Array(length, opt_byteOffset, opt_length) {}

/** @type {number} */
Int32Array.BYTES_PER_ELEMENT;

/** @type {number} */
Int32Array.prototype.BYTES_PER_ELEMENT;

/** @type {number} */
Int32Array.prototype.length;

/**
 * @param {ArrayBufferView|Array.<number>} array
 * @param {number=} opt_offset
 */
Int32Array.prototype.set = function(array, opt_offset) {};

/**
 * @param {number} begin
 * @param {number=} opt_end
 * @return {!Int32Array}
 * @nosideeffects
 */
Int32Array.prototype.subarray = function(begin, opt_end) {};


/**
 * @param {number|ArrayBufferView|Array.<number>|ArrayBuffer} length or array
 *     or buffer
 * @param {number=} opt_byteOffset
 * @param {number=} opt_length
 * @extends {ArrayBufferView}
 * @constructor
 * @noalias
 * @throws {Error}
 * @modifies {arguments}
 */
function Uint32Array(length, opt_byteOffset, opt_length) {}

/** @type {number} */
Uint32Array.BYTES_PER_ELEMENT;

/** @type {number} */
Uint32Array.prototype.BYTES_PER_ELEMENT;

/** @type {number} */
Uint32Array.prototype.length;

/**
 * @param {ArrayBufferView|Array.<number>} array
 * @param {number=} opt_offset
 */
Uint32Array.prototype.set = function(array, opt_offset) {};

/**
 * @param {number} begin
 * @param {number=} opt_end
 * @return {!Uint32Array}
 * @nosideeffects
 */
Uint32Array.prototype.subarray = function(begin, opt_end) {};


/**
 * @param {number|ArrayBufferView|Array.<number>|ArrayBuffer} length or array
 *     or buffer
 * @param {number=} opt_byteOffset
 * @param {number=} opt_length
 * @extends {ArrayBufferView}
 * @constructor
 * @noalias
 * @throws {Error}
 * @modifies {arguments}
 */
function Float32Array(length, opt_byteOffset, opt_length) {}

/** @type {number} */
Float32Array.BYTES_PER_ELEMENT;

/** @type {number} */
Float32Array.prototype.BYTES_PER_ELEMENT;

/** @type {number} */
Float32Array.prototype.length;

/**
 * @param {ArrayBufferView|Array.<number>} array
 * @param {number=} opt_offset
 */
Float32Array.prototype.set = function(array, opt_offset) {};

/**
 * @param {number} begin
 * @param {number=} opt_end
 * @return {!Float32Array}
 * @nosideeffects
 */
Float32Array.prototype.subarray = function(begin, opt_end) {};


/**
 * @param {number|ArrayBufferView|Array.<number>|ArrayBuffer} length or array
 *     or buffer
 * @param {number=} opt_byteOffset
 * @param {number=} opt_length
 * @extends {ArrayBufferView}
 * @constructor
 * @noalias
 * @throws {Error}
 * @modifies {arguments}
 */
function Float64Array(length, opt_byteOffset, opt_length) {}

/** @type {number} */
Float64Array.BYTES_PER_ELEMENT;

/** @type {number} */
Float64Array.prototype.BYTES_PER_ELEMENT;

/** @type {number} */
Float64Array.prototype.length;

/**
 * @param {ArrayBufferView|Array.<number>} array
 * @param {number=} opt_offset
 */
Float64Array.prototype.set = function(array, opt_offset) {};

/**
 * @param {number} begin
 * @param {number=} opt_end
 * @return {!Float64Array}
 * @nosideeffects
 */
Float64Array.prototype.subarray = function(begin, opt_end) {};


/**
 * @param {ArrayBuffer} buffer
 * @param {number=} opt_byteOffset
 * @param {number=} opt_byteLength
 * @extends {ArrayBufferView}
 * @constructor
 * @noalias
 * @throws {Error}
 * @nosideeffects
 * @see https://developer.mozilla.org/en-US/docs/Web/JavaScript/Typed_arrays/DataView
 */
function DataView(buffer, opt_byteOffset, opt_byteLength) {}

/**
 * @param {number} byteOffset
 * @return {number}
 * @throws {Error}
 * @nosideeffects
 */
DataView.prototype.getInt8 = function(byteOffset) {};

/**
 * @param {number} byteOffset
 * @return {number}
 * @throws {Error}
 * @nosideeffects
 */
DataView.prototype.getUint8 = function(byteOffset) {};

/**
 * @param {number} byteOffset
 * @param {boolean=} opt_littleEndian
 * @return {number}
 * @throws {Error}
 * @nosideeffects
 */
DataView.prototype.getInt16 = function(byteOffset, opt_littleEndian) {};

/**
 * @param {number} byteOffset
 * @param {boolean=} opt_littleEndian
 * @return {number}
 * @throws {Error}
 * @nosideeffects
 */
DataView.prototype.getUint16 = function(byteOffset, opt_littleEndian) {};

/**
 * @param {number} byteOffset
 * @param {boolean=} opt_littleEndian
 * @return {number}
 * @throws {Error}
 * @nosideeffects
 */
DataView.prototype.getInt32 = function(byteOffset, opt_littleEndian) {};

/**
 * @param {number} byteOffset
 * @param {boolean=} opt_littleEndian
 * @return {number}
 * @throws {Error}
 * @nosideeffects
 */
DataView.prototype.getUint32 = function(byteOffset, opt_littleEndian) {};

/**
 * @param {number} byteOffset
 * @param {boolean=} opt_littleEndian
 * @return {number}
 * @throws {Error}
 * @nosideeffects
 */
DataView.prototype.getFloat32 = function(byteOffset, opt_littleEndian) {};

/**
 * @param {number} byteOffset
 * @param {boolean=} opt_littleEndian
 * @return {number}
 * @throws {Error}
 * @nosideeffects
 */
DataView.prototype.getFloat64 = function(byteOffset, opt_littleEndian) {};

/**
 * @param {number} byteOffset
 * @param {number} value
 * @throws {Error}
 */
DataView.prototype.setInt8 = function(byteOffset, value) {};

/**
 * @param {number} byteOffset
 * @param {number} value
 * @throws {Error}
 */
DataView.prototype.setUint8 = function(byteOffset, value) {};

/**
 * @param {number} byteOffset
 * @param {number} value
 * @param {boolean=} opt_littleEndian
 * @throws {Error}
 */
DataView.prototype.setInt16 = function(byteOffset, value, opt_littleEndian) {};

/**
 * @param {number} byteOffset
 * @param {number} value
 * @param {boolean=} opt_littleEndian
 * @throws {Error}
 */
DataView.prototype.setUint16 = function(byteOffset, value, opt_littleEndian) {};

/**
 * @param {number} byteOffset
 * @param {number} value
 * @param {boolean=} opt_littleEndian
 * @throws {Error}
 */
DataView.prototype.setInt32 = function(byteOffset, value, opt_littleEndian) {};

/**
 * @param {number} byteOffset
 * @param {number} value
 * @param {boolean=} opt_littleEndian
 * @throws {Error}
 */
DataView.prototype.setUint32 = function(byteOffset, value, opt_littleEndian) {};

/**
 * @param {number} byteOffset
 * @param {number} value
 * @param {boolean=} opt_littleEndian
 * @throws {Error}
 */
DataView.prototype.setFloat32 = function(
    byteOffset, value, opt_littleEndian) {};

/**
 * @param {number} byteOffset
 * @param {number} value
 * @param {boolean=} opt_littleEndian
 * @throws {Error}
 */
DataView.prototype.setFloat64 = function(
    byteOffset, value, opt_littleEndian) {};


/**
 * @see https://github.com/promises-aplus/promises-spec
 * @typedef {{then: !Function}}
 */
var Thenable;


/**
 * This is not an official DOM interface. It is used to add generic typing
 * and respective type inference where available.
 * {@see goog.Thenable} inherits from this making all promises
 * interoperate.
 * @interface
 * @template TYPE
 */
var IThenable = function() {};


/**
 * @param {?(function(TYPE):
 *             (RESULT|IThenable.<RESULT>|Thenable))=} opt_onFulfilled
 * @param {?(function(*): *)=} opt_onRejected
 * @return {!IThenable.<RESULT>}
 * @template RESULT
 */
IThenable.prototype.then = function(opt_onFulfilled, opt_onRejected) {};


/**
 * @see https://people.mozilla.org/~jorendorff/es6-draft.html#sec-promise-objects
 * @param {function(
 *             function((TYPE|IThenable.<TYPE>|Thenable|null)=),
 *             function(*=))} resolver
 * @constructor
 * @implements {IThenable.<TYPE>}
 * @template TYPE
 */
var Promise = function(resolver) {};


/**
 * @param {(TYPE|IThenable.<TYPE>)=} opt_value
 * @return {!Promise.<TYPE>}
 * @template TYPE
 */
Promise.resolve = function(opt_value) {};


/**
 * @param {*=} opt_error
 * @return {!Promise.<?>}
 */
Promise.reject = function(opt_error) {};


/**
 * @template T
 * @see https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Promise
 * @param {!Array<T|!Promise<T>>} iterable
 * @return {!Promise<!Array<T>>}
 */
Promise.all = function(iterable) {};


/**
 * @template T
 * @see https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Promise
 * @param {!Array.<T>} iterable
 * @return {!Promise.<T>}
 */
Promise.race = function(iterable) {};


/**
 * @param {?(function(TYPE):
 *             (RESULT|IThenable.<RESULT>|Thenable))=} opt_onFulfilled
 * @param {?(function(*): *)=} opt_onRejected
 * @return {!Promise.<RESULT>}
 * @template RESULT
 * @override
 */
Promise.prototype.then = function(opt_onFulfilled, opt_onRejected) {};


/**
 * @param {function(*): RESULT} onRejected
 * @return {!Promise.<RESULT>}
 * @template RESULT
 */
Promise.prototype.catch = function(onRejected) {};
