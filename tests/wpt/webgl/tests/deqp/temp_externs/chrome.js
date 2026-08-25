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
 * @fileoverview Definitions for globals in Chrome.  This file describes the
 * externs API for the chrome.* object when running in a normal browser
 * context.  For APIs available in Chrome Extensions, see chrome_extensions.js
 * in this directory.
 * @externs
 */


/**
 * namespace
 * @const
 */
var chrome = {};



/**
 * Returns an object representing current load times. Note that the properties
 * on the object do not change and the function must be called again to get
 * up-to-date data.
 *
 * @see http://goto.google.com/chromeloadtimesextension
 *
 * @return {!ChromeLoadTimes}
 */
chrome.loadTimes = function() {};



/**
 * The data object given by chrome.loadTimes().
 * @constructor
 */
function ChromeLoadTimes() {}


/** @type {number} */
ChromeLoadTimes.prototype.requestTime;


/** @type {number} */
ChromeLoadTimes.prototype.startLoadTime;


/** @type {number} */
ChromeLoadTimes.prototype.commitLoadTime;


/** @type {number} */
ChromeLoadTimes.prototype.finishDocumentLoadTime;


/** @type {number} */
ChromeLoadTimes.prototype.finishLoadTime;


/** @type {number} */
ChromeLoadTimes.prototype.firstPaintTime;


/** @type {number} */
ChromeLoadTimes.prototype.firstPaintAfterLoadTime;


/** @type {number} */
ChromeLoadTimes.prototype.navigationType;


/**
 * True iff the resource was fetched over SPDY.
 * @type {boolean}
 */
ChromeLoadTimes.prototype.wasFetchedViaSpdy;


/** @type {boolean} */
ChromeLoadTimes.prototype.wasNpnNegotiated;


/** @type {string} */
ChromeLoadTimes.prototype.npnNegotiatedProtocol;


/** @type {boolean} */
ChromeLoadTimes.prototype.wasAlternateProtocolAvailable;


/** @type {string} */
ChromeLoadTimes.prototype.connectionInfo;


/**
 * Returns an object containing timing information.
 * @return {!ChromeCsiInfo}
 */
chrome.csi = function() {};



/**
 * The data object given by chrome.csi().
 * @constructor
 */
function ChromeCsiInfo() {}


/**
 * Same as chrome.loadTimes().requestTime, if defined.
 * Otherwise, gives the same value as chrome.loadTimes().startLoadTime.
 * In milliseconds, truncated.
 * @type {number}
 */
ChromeCsiInfo.prototype.startE;


/**
 * Same as chrome.loadTimes().finishDocumentLoadTime but in milliseconds and
 * truncated.
 * @type {number}
 */
ChromeCsiInfo.prototype.onloadT;


/**
 * The time since startE in milliseconds.
 * @type {number}
 */
ChromeCsiInfo.prototype.pageT;


/** @type {number} */
ChromeCsiInfo.prototype.tran;


/**
 * @param {string|!ArrayBuffer|!Object} message
 * @see https://developers.google.com/native-client/devguide/tutorial
 */
HTMLEmbedElement.prototype.postMessage = function(message) {};
