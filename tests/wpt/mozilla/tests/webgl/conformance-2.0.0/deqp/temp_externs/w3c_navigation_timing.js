/*
 * Copyright 2011 The Closure Compiler Authors
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
 * @fileoverview Definitions for W3C's Navigation Timing specification.
 *
 * Created from
 * @see http://dvcs.w3.org/hg/webperf/raw-file/tip/specs/NavigationTiming/Overview.html
 * @see http://w3c-test.org/webperf/specs/ResourceTiming
 * @see http://www.w3.org/TR/performance-timeline
 *
 * @externs
 */

/** @constructor */
function PerformanceTiming() {}
/** @type {number} */ PerformanceTiming.prototype.navigationStart;
/** @type {number} */ PerformanceTiming.prototype.unloadEventStart;
/** @type {number} */ PerformanceTiming.prototype.unloadEventEnd;
/** @type {number} */ PerformanceTiming.prototype.redirectStart;
/** @type {number} */ PerformanceTiming.prototype.redirectEnd;
/** @type {number} */ PerformanceTiming.prototype.fetchStart;
/** @type {number} */ PerformanceTiming.prototype.domainLookupStart;
/** @type {number} */ PerformanceTiming.prototype.domainLookupEnd;
/** @type {number} */ PerformanceTiming.prototype.connectStart;
/** @type {number} */ PerformanceTiming.prototype.connectEnd;
/** @type {number} */ PerformanceTiming.prototype.secureConnectionStart;
/** @type {number} */ PerformanceTiming.prototype.requestStart;
/** @type {number} */ PerformanceTiming.prototype.responseStart;
/** @type {number} */ PerformanceTiming.prototype.responseEnd;
/** @type {number} */ PerformanceTiming.prototype.domLoading;
/** @type {number} */ PerformanceTiming.prototype.domInteractive;
/** @type {number} */ PerformanceTiming.prototype.domContentLoadedEventStart;
/** @type {number} */ PerformanceTiming.prototype.domContentLoadedEventEnd;
/** @type {number} */ PerformanceTiming.prototype.domComplete;
/** @type {number} */ PerformanceTiming.prototype.loadEventStart;
/** @type {number} */ PerformanceTiming.prototype.loadEventEnd;

/** @constructor */
function PerformanceEntry() {}
/** @type {string} */ PerformanceEntry.prototype.name;
/** @type {string} */ PerformanceEntry.prototype.entryType;
/** @type {number} */ PerformanceEntry.prototype.startTime;
/** @type {number} */ PerformanceEntry.prototype.duration;

/**
 * @constructor
 * @extends {PerformanceEntry}
 */
function PerformanceResourceTiming() {}
/** @type {number} */ PerformanceResourceTiming.prototype.redirectStart;
/** @type {number} */ PerformanceResourceTiming.prototype.redirectEnd;
/** @type {number} */ PerformanceResourceTiming.prototype.fetchStart;
/** @type {number} */ PerformanceResourceTiming.prototype.domainLookupStart;
/** @type {number} */ PerformanceResourceTiming.prototype.domainLookupEnd;
/** @type {number} */ PerformanceResourceTiming.prototype.connectStart;
/** @type {number} */ PerformanceResourceTiming.prototype.connectEnd;
/** @type {number} */
PerformanceResourceTiming.prototype.secureConnectionStart;
/** @type {number} */ PerformanceResourceTiming.prototype.requestStart;
/** @type {number} */ PerformanceResourceTiming.prototype.responseStart;
/** @type {number} */ PerformanceResourceTiming.prototype.responseEnd;
/** @type {string} */ PerformanceResourceTiming.prototype.initiatorType;

/** @constructor */
function PerformanceNavigation() {}
/** @type {number} */ PerformanceNavigation.prototype.TYPE_NAVIGATE = 0;
/** @type {number} */ PerformanceNavigation.prototype.TYPE_RELOAD = 1;
/** @type {number} */ PerformanceNavigation.prototype.TYPE_BACK_FORWARD = 2;
/** @type {number} */ PerformanceNavigation.prototype.TYPE_RESERVED = 255;
/** @type {number} */ PerformanceNavigation.prototype.type;
/** @type {number} */ PerformanceNavigation.prototype.redirectCount;

// Only available in WebKit, and only with the --enable-memory-info flag.
/** @constructor */
function PerformanceMemory() {}
/** @type {number} */ PerformanceMemory.prototype.jsHeapSizeLimit;
/** @type {number} */ PerformanceMemory.prototype.totalJSHeapSize;
/** @type {number} */ PerformanceMemory.prototype.usedJSHeapSize;

/** @constructor */
function Performance() {}
/** @type {PerformanceTiming} */ Performance.prototype.timing;
/** @type {PerformanceNavigation} */ Performance.prototype.navigation;

/**
 * Clears the buffer used to store the current list of
 * PerformanceResourceTiming resources.
 * @return {undefined}
 */
Performance.prototype.clearResourceTimings = function() {};

/**
 * Clear out the buffer of performance timing events for webkit browsers.
 * @return {undefined}
 */
Performance.prototype.webkitClearResourceTimings = function() {};

/**
 * Set the maximum number of PerformanceResourceTiming resources that may be
 * stored in the buffer.
 * @param {number} maxSize
 */
Performance.prototype.setResourceTimingBufferSize = function(maxSize) {};

/**
 * @return {Array.<PerformanceEntry>} A copy of the PerformanceEntry list,
 *     in chronological order with respect to startTime.
 * @nosideeffects
 */
Performance.prototype.getEntries = function() {};

/**
 * @param {string} entryType Only return {@code PerformanceEntry}s with this
 *     entryType.
 * @return {Array.<PerformanceEntry>} A copy of the PerformanceEntry list,
 *     in chronological order with respect to startTime.
 * @nosideeffects
 */
Performance.prototype.getEntriesByType = function(entryType) {};

/**
 * @param {string} name Only return {@code PerformanceEntry}s with this name.
 * @param {string=} opt_entryType Only return {@code PerformanceEntry}s with
 *     this entryType.
 * @return {Array.<PerformanceEntry>} PerformanceEntry list in chronological
 *     order with respect to startTime.
 * @nosideeffects
 */
Performance.prototype.getEntriesByName = function(name, opt_entryType) {};

// Only available in WebKit, and only with the --enable-memory-info flag.
/** @type {PerformanceMemory} */ Performance.prototype.memory;

/**
 * @return {number}
 * @nosideeffects
 */
Performance.prototype.now = function() {};

/**
 * @return {number}
 * @nosideeffects
 */
Performance.prototype.webkitNow = function() {};

/** @type {Performance} */
Window.prototype.performance;
