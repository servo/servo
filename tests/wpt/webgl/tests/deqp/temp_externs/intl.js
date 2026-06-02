/*
 * Copyright 2013 The Closure Compiler Authors
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
 * @fileoverview Definitions for the JS Internationalization API as defined in
 * http://www.ecma-international.org/ecma-402/1.0/
 *
 * @externs
 */

/** @const */
var Intl = {};

/**
 * NOTE: this API is not from ecma402 and is subject to change.
 * @param {string|Array.<string>=} opt_locales
 * @param {{type: (string|undefined)}=}
 *         opt_options
 * @constructor
 */
Intl.v8BreakIterator = function(opt_locales, opt_options) {};

/**
 * @param {string} text
 */
Intl.v8BreakIterator.prototype.adoptText = function(text) {};

/**
 * @return {string}
 */
Intl.v8BreakIterator.prototype.breakType = function() {};

/**
 * @return {number}
 */
Intl.v8BreakIterator.prototype.current = function() {};

/**
 * @return {number}
 */
Intl.v8BreakIterator.prototype.first = function() {};

/**
 * @return {number}
 */
Intl.v8BreakIterator.prototype.next = function() {};

/**
 * @constructor
 * @param {string|Array.<string>=} opt_locales
 * @param {{usage: (string|undefined), localeMatcher: (string|undefined),
 *     sensitivity: (string|undefined), ignorePunctuation: (boolean|undefined),
 *     numeric: (boolean|undefined), caseFirst: (string|undefined)}=}
 *         opt_options
 */
Intl.Collator = function(opt_locales, opt_options) {};

/**
 * @param {Array.<string>} locales
 * @param {{localeMatcher: (string|undefined)}=} opt_options
 */
Intl.Collator.supportedLocalesOf = function(locales, opt_options) {};

/**
 * @param {string} arg1
 * @param {string} arg2
 * @return {number}
 */
Intl.Collator.prototype.compare = function(arg1, arg2) {};

/**
 * @return {{locale: string, usage: string, sensitivity: string,
 *     ignorePunctuation: boolean, collation: string, numeric: boolean,
 *     caseFirst: string}}
 */
Intl.Collator.prototype.resolvedOptions = function() {};

/**
 * @constructor
 * @param {string|Array.<string>=} opt_locales
 * @param {{localeMatcher: (string|undefined), useGrouping: (boolean|undefined),
 *     numberingSystem: (string|undefined), style: (string|undefined),
 *     currency: (string|undefined), currencyDisplay: (string|undefined),
 *     minimumIntegerDigits: (number|undefined),
 *     minimumFractionDigits: (number|undefined),
 *     maximumFractionDigits: (number|undefined),
 *     minimumSignificantDigits: (number|undefined),
 *     maximumSignificantDigits: (number|undefined)}=}
 *         opt_options
 */
Intl.NumberFormat = function(opt_locales, opt_options) {};

/**
 * @param {Array.<string>} locales
 * @param {{localeMatcher: (string|undefined)}=} opt_options
 */
Intl.NumberFormat.supportedLocalesOf = function(locales, opt_options) {};

/**
 * @param {number} num
 * @return {string}
 */
Intl.NumberFormat.prototype.format = function(num) {};

/**
 * @return {{locale: string, numberingSystem: string, style: string,
 *     currency: (string|undefined), currencyDisplay: (string|undefined),
 *     minimumIntegerDigits: number, minimumFractionDigits: number,
 *     maximumFractionDigits: number, minimumSignificantDigits: number,
 *     maximumSignificantDigits: number, useGrouping: boolean}}
 */
Intl.NumberFormat.prototype.resolvedOptions = function() {};

/**
 * @constructor
 * @param {string|Array.<string>=} opt_locales
 * @param {{localeMatcher: (string|undefined),
 *    formatMatcher: (string|undefined), calendar: (string|undefined),
 *    numberingSystem: (string|undefined), tz: (string|undefined),
 *    weekday: (string|undefined), era: (string|undefined),
 *    year: (string|undefined), month: (string|undefined),
 *    day: (string|undefined), hour: (string|undefined),
 *    minute: (string|undefined), second: (string|undefined),
 *    timeZoneName: (string|undefined), hour12: (boolean|undefined)}=}
 *        opt_options
 */
Intl.DateTimeFormat = function(opt_locales, opt_options) {};

/**
 * @param {Array.<string>} locales
 * @param {{localeMatcher: string}=} opt_options
 */
Intl.DateTimeFormat.supportedLocalesOf = function(locales, opt_options) {};

/**
 * @param {number} date
 * @return {string}
 */
Intl.DateTimeFormat.prototype.format = function(date) {};

/**
 * @return {{locale: string, calendar: string, numberingSystem: string,
 *    timeZone: (string|undefined), weekday: (string|undefined),
 *    era: (string|undefined), year: (string|undefined),
 *    month: (string|undefined), day: (string|undefined),
 *    hour: (string|undefined), minute: (string|undefined),
 *    second: (string|undefined), timeZoneName: (string|undefined),
 *    hour12: (boolean|undefined)}}
 */
Intl.DateTimeFormat.prototype.resolvedOptions = function() {};
