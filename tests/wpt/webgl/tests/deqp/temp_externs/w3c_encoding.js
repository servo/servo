/*
 * Copyright 2015 The Closure Compiler Authors
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
 * @fileoverview Definitions for W3C's Encoding specification
 *     https://encoding.spec.whatwg.org
 * @externs
 */

/**
 * @constructor
 * @param {string=} encoding
 * @param {Object=} options
 */
function TextDecoder(encoding, options) {}

/** @type {string} **/ TextDecoder.prototype.encoding;
/** @type {boolean} **/ TextDecoder.prototype.fatal;
/** @type {boolean} **/ TextDecoder.prototype.ignoreBOM;

/**
 * @param {!Uint8Array} input
 * @param {Object=} options
 * @return {string}
 */
TextDecoder.prototype.decode = function decode(input, options) {};

/**
 * @constructor
 * @param {string=} encoding
 * @param {Object=} options
 */
function TextEncoder(encoding, options) {}

/** @type {string} **/ TextEncoder.prototype.encoding;

/**
 * @param {string} input
 * @return {!Uint8Array}
 */
TextEncoder.prototype.encode = function(input) {};
