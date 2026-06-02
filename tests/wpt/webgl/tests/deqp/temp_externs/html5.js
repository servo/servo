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
 * @fileoverview Definitions for all the extensions over the
 *  W3C's DOM3 specification in HTML5. This file depends on
 *  w3c_dom3.js. The whole file has been fully type annotated.
 *
 *  @see http://www.whatwg.org/specs/web-apps/current-work/multipage/index.html
 *  @see http://dev.w3.org/html5/spec/Overview.html
 *
 *  This also includes Typed Array definitions from
 *  http://www.khronos.org/registry/typedarray/specs/latest/
 *
 *  This relies on w3c_event.js being included first.
 *
 * @externs
 */


/**
 * Note: In IE, the contains() method only exists on Elements, not Nodes.
 * Therefore, it is recommended that you use the Conformance framework to
 * prevent calling this on Nodes which are not Elements.
 * @see https://connect.microsoft.com/IE/feedback/details/780874/node-contains-is-incorrect
 *
 * @param {Node} n The node to check
 * @return {boolean} If 'n' is this Node, or is contained within this Node.
 * @see https://developer.mozilla.org/en-US/docs/Web/API/Node.contains
 * @nosideeffects
 */
Node.prototype.contains = function(n) {};


/**
 * @constructor
 * @see http://www.whatwg.org/specs/web-apps/current-work/multipage/the-canvas-element.html#the-canvas-element
 * @extends {HTMLElement}
 */
function HTMLCanvasElement() {}

/** @type {number} */
HTMLCanvasElement.prototype.width;

/** @type {number} */
HTMLCanvasElement.prototype.height;

/**
 * @param {string=} opt_type
 * @param {...*} var_args
 * @return {string}
 * @throws {Error}
 * @nosideeffects
 */
HTMLCanvasElement.prototype.toDataURL = function(opt_type, var_args) {};

/**
 * @param {string} contextId
 * @param {Object=} opt_args
 * @return {Object}
 */
HTMLCanvasElement.prototype.getContext = function(contextId, opt_args) {};

/**
 * @constructor
 * @see http://www.w3.org/TR/2dcontext/#canvasrenderingcontext2d
 */
function CanvasRenderingContext2D() {}

/** @type {HTMLCanvasElement} */
CanvasRenderingContext2D.prototype.canvas;

/**
 * @return {undefined}
 */
CanvasRenderingContext2D.prototype.save = function() {};

/**
 * @return {undefined}
 */
CanvasRenderingContext2D.prototype.restore = function() {};

/**
 * @param {number} x
 * @param {number} y
 * @return {undefined}
 */
CanvasRenderingContext2D.prototype.scale = function(x, y) {};

/**
 * @param {number} angle
 * @return {undefined}
 */
CanvasRenderingContext2D.prototype.rotate = function(angle) {};

/**
 * @param {number} x
 * @param {number} y
 * @return {undefined}
 */
CanvasRenderingContext2D.prototype.translate = function(x, y) {};

/**
 * @param {number} m11
 * @param {number} m12
 * @param {number} m21
 * @param {number} m22
 * @param {number} dx
 * @param {number} dy
 * @return {undefined}
 */
CanvasRenderingContext2D.prototype.transform = function(
    m11, m12, m21, m22, dx, dy) {};

/**
 * @param {number} m11
 * @param {number} m12
 * @param {number} m21
 * @param {number} m22
 * @param {number} dx
 * @param {number} dy
 * @return {undefined}
 */
CanvasRenderingContext2D.prototype.setTransform = function(
    m11, m12, m21, m22, dx, dy) {};

/**
 * @param {number} x0
 * @param {number} y0
 * @param {number} x1
 * @param {number} y1
 * @return {CanvasGradient}
 * @throws {Error}
 * @nosideeffects
 */
CanvasRenderingContext2D.prototype.createLinearGradient = function(
    x0, y0, x1, y1) {};

/**
 * @param {number} x0
 * @param {number} y0
 * @param {number} r0
 * @param {number} x1
 * @param {number} y1
 * @param {number} r1
 * @return {CanvasGradient}
 * @throws {Error}
 * @nosideeffects
 */
CanvasRenderingContext2D.prototype.createRadialGradient = function(
    x0, y0, r0, x1, y1, r1) {};

/**
 * @param {HTMLImageElement|HTMLCanvasElement} image
 * @param {string} repetition
 * @return {CanvasPattern}
 * @throws {Error}
 * @nosideeffects
 */
CanvasRenderingContext2D.prototype.createPattern = function(
    image, repetition) {};

/**
 * @param {number} x
 * @param {number} y
 * @param {number} w
 * @param {number} h
 * @return {undefined}
 */
CanvasRenderingContext2D.prototype.clearRect = function(x, y, w, h) {};

/**
 * @param {number} x
 * @param {number} y
 * @param {number} w
 * @param {number} h
 * @return {undefined}
 */
CanvasRenderingContext2D.prototype.fillRect = function(x, y, w, h) {};

/**
 * @param {number} x
 * @param {number} y
 * @param {number} w
 * @param {number} h
 * @return {undefined}
 */
CanvasRenderingContext2D.prototype.strokeRect = function(x, y, w, h) {};

/**
 * @return {undefined}
 */
CanvasRenderingContext2D.prototype.beginPath = function() {};

/**
 * @return {undefined}
 */
CanvasRenderingContext2D.prototype.closePath = function() {};

/**
 * @param {number} x
 * @param {number} y
 * @return {undefined}
 */
CanvasRenderingContext2D.prototype.moveTo = function(x, y) {};

/**
 * @param {number} x
 * @param {number} y
 * @return {undefined}
 */
CanvasRenderingContext2D.prototype.lineTo = function(x, y) {};

/**
 * @param {number} cpx
 * @param {number} cpy
 * @param {number} x
 * @param {number} y
 * @return {undefined}
 */
CanvasRenderingContext2D.prototype.quadraticCurveTo = function(
    cpx, cpy, x, y) {};

/**
 * @param {number} cp1x
 * @param {number} cp1y
 * @param {number} cp2x
 * @param {number} cp2y
 * @param {number} x
 * @param {number} y
 * @return {undefined}
 */
CanvasRenderingContext2D.prototype.bezierCurveTo = function(
    cp1x, cp1y, cp2x, cp2y, x, y) {};

/**
 * @param {number} x1
 * @param {number} y1
 * @param {number} x2
 * @param {number} y2
 * @param {number} radius
 * @return {undefined}
 */
CanvasRenderingContext2D.prototype.arcTo = function(x1, y1, x2, y2, radius) {};

/**
 * @param {number} x
 * @param {number} y
 * @param {number} w
 * @param {number} h
 * @return {undefined}
 */
CanvasRenderingContext2D.prototype.rect = function(x, y, w, h) {};

/**
 * @param {number} x
 * @param {number} y
 * @param {number} radius
 * @param {number} startAngle
 * @param {number} endAngle
 * @param {boolean=} opt_anticlockwise
 * @return {undefined}
 */
CanvasRenderingContext2D.prototype.arc = function(
    x, y, radius, startAngle, endAngle, opt_anticlockwise) {};

/**
 * @return {undefined}
 */
CanvasRenderingContext2D.prototype.fill = function() {};

/**
 * @return {undefined}
 */
CanvasRenderingContext2D.prototype.stroke = function() {};

/**
 * @return {undefined}
 */
CanvasRenderingContext2D.prototype.clip = function() {};

/**
 * @param {number} x
 * @param {number} y
 * @return {boolean}
 * @nosideeffects
 */
CanvasRenderingContext2D.prototype.isPointInPath = function(x, y) {};

/**
 * @param {string} text
 * @param {number} x
 * @param {number} y
 * @param {number=} opt_maxWidth
 * @return {undefined}
 */
CanvasRenderingContext2D.prototype.fillText = function(
    text, x, y, opt_maxWidth) {};

/**
 * @param {string} text
 * @param {number} x
 * @param {number} y
 * @param {number=} opt_maxWidth
 * @return {undefined}
 */
CanvasRenderingContext2D.prototype.strokeText = function(
    text, x, y, opt_maxWidth) {};

/**
 * @param {string} text
 * @return {TextMetrics}
 * @nosideeffects
 */
CanvasRenderingContext2D.prototype.measureText = function(text) {};

/**
 * @param {HTMLImageElement|HTMLCanvasElement|Image|HTMLVideoElement} image
 * @param {number} dx Destination x coordinate.
 * @param {number} dy Destination y coordinate.
 * @param {number=} opt_dw Destination box width.  Defaults to the image width.
 * @param {number=} opt_dh Destination box height.
 *     Defaults to the image height.
 * @param {number=} opt_sx Source box x coordinate.  Used to select a portion of
 *     the source image to draw.  Defaults to 0.
 * @param {number=} opt_sy Source box y coordinate.  Used to select a portion of
 *     the source image to draw.  Defaults to 0.
 * @param {number=} opt_sw Source box width.  Used to select a portion of
 *     the source image to draw.  Defaults to the full image width.
 * @param {number=} opt_sh Source box height.  Used to select a portion of
 *     the source image to draw.  Defaults to the full image height.
 * @return {undefined}
 */
CanvasRenderingContext2D.prototype.drawImage = function(
    image, dx, dy, opt_dw, opt_dh, opt_sx, opt_sy, opt_sw, opt_sh) {};

/**
 * @param {number} sw
 * @param {number} sh
 * @return {ImageData}
 * @nosideeffects
 */
CanvasRenderingContext2D.prototype.createImageData = function(sw, sh) {};

/**
 * @param {number} sx
 * @param {number} sy
 * @param {number} sw
 * @param {number} sh
 * @return {ImageData}
 * @throws {Error}
 * @nosideeffects
 */
CanvasRenderingContext2D.prototype.getImageData = function(sx, sy, sw, sh) {};

/**
 * @param {ImageData} imagedata
 * @param {number} dx
 * @param {number} dy
 * @param {number=} opt_dirtyX
 * @param {number=} opt_dirtyY
 * @param {number=} opt_dirtyWidth
 * @param {number=} opt_dirtyHeight
 * @return {undefined}
 */
CanvasRenderingContext2D.prototype.putImageData = function(imagedata, dx, dy,
    opt_dirtyX, opt_dirtyY, opt_dirtyWidth, opt_dirtyHeight) {};

/**
 * Note: WebKit only
 * @param {number|string=} opt_a
 * @param {number=} opt_b
 * @param {number=} opt_c
 * @param {number=} opt_d
 * @param {number=} opt_e
 * @see http://developer.apple.com/library/safari/#documentation/appleapplications/reference/WebKitDOMRef/CanvasRenderingContext2D_idl/Classes/CanvasRenderingContext2D/index.html
 * @return {undefined}
 */
CanvasRenderingContext2D.prototype.setFillColor;

/**
 * Note: WebKit only
 * @param {number|string=} opt_a
 * @param {number=} opt_b
 * @param {number=} opt_c
 * @param {number=} opt_d
 * @param {number=} opt_e
 * @see http://developer.apple.com/library/safari/#documentation/appleapplications/reference/WebKitDOMRef/CanvasRenderingContext2D_idl/Classes/CanvasRenderingContext2D/index.html
 * @return {undefined}
 */
CanvasRenderingContext2D.prototype.setStrokeColor;

/**
 * @return {Array.<number>}
 */
CanvasRenderingContext2D.prototype.getLineDash;

/**
 * @param {Array.<number>} segments
 * @return {undefined}
 */
CanvasRenderingContext2D.prototype.setLineDash;

/** @type {string} */
CanvasRenderingContext2D.prototype.fillColor;

/**
 * @type {string}
 * @implicitCast
 */
CanvasRenderingContext2D.prototype.fillStyle;

/** @type {string} */
CanvasRenderingContext2D.prototype.font;

/** @type {number} */
CanvasRenderingContext2D.prototype.globalAlpha;

/** @type {string} */
CanvasRenderingContext2D.prototype.globalCompositeOperation;

/** @type {number} */
CanvasRenderingContext2D.prototype.lineWidth;

/** @type {string} */
CanvasRenderingContext2D.prototype.lineCap;

/** @type {string} */
CanvasRenderingContext2D.prototype.lineJoin;

/** @type {number} */
CanvasRenderingContext2D.prototype.miterLimit;

/** @type {number} */
CanvasRenderingContext2D.prototype.shadowBlur;

/** @type {string} */
CanvasRenderingContext2D.prototype.shadowColor;

/** @type {number} */
CanvasRenderingContext2D.prototype.shadowOffsetX;

/** @type {number} */
CanvasRenderingContext2D.prototype.shadowOffsetY;

/**
 * @type {string}
 * @implicitCast
 */
CanvasRenderingContext2D.prototype.strokeStyle;

/** @type {string} */
CanvasRenderingContext2D.prototype.strokeColor;

/** @type {string} */
CanvasRenderingContext2D.prototype.textAlign;

/** @type {string} */
CanvasRenderingContext2D.prototype.textBaseline;

/**
 * @constructor
 */
function CanvasGradient() {}

/**
 * @param {number} offset
 * @param {string} color
 * @return {undefined}
 */
CanvasGradient.prototype.addColorStop = function(offset, color) {};

/**
 * @constructor
 */
function CanvasPattern() {}

/**
 * @constructor
 */
function TextMetrics() {}

/** @type {number} */
TextMetrics.prototype.width;

/**
 * @constructor
 */
function ImageData() {}

/** @type {Uint8ClampedArray} */
ImageData.prototype.data;

/** @type {number} */
ImageData.prototype.width;

/** @type {number} */
ImageData.prototype.height;

/**
 * @constructor
 */
function ClientInformation() {}

/** @type {boolean} */
ClientInformation.prototype.onLine;

/**
 * @param {string} protocol
 * @param {string} uri
 * @param {string} title
 * @return {undefined}
 */
ClientInformation.prototype.registerProtocolHandler = function(
    protocol, uri, title) {};

/**
 * @param {string} mimeType
 * @param {string} uri
 * @param {string} title
 * @return {undefined}
 */
ClientInformation.prototype.registerContentHandler = function(
    mimeType, uri, title) {};

// HTML5 Database objects
/**
 * @constructor
 */
function Database() {}

/**
 * @type {string}
 */
Database.prototype.version;

/**
 * @param {function(!SQLTransaction) : void} callback
 * @param {(function(!SQLError) : void)=} opt_errorCallback
 * @param {Function=} opt_Callback
 */
Database.prototype.transaction = function(
    callback, opt_errorCallback, opt_Callback) {};

/**
 * @param {function(!SQLTransaction) : void} callback
 * @param {(function(!SQLError) : void)=} opt_errorCallback
 * @param {Function=} opt_Callback
 */
Database.prototype.readTransaction = function(
    callback, opt_errorCallback, opt_Callback) {};

/**
 * @param {string} oldVersion
 * @param {string} newVersion
 * @param {function(!SQLTransaction) : void} callback
 * @param {function(!SQLError) : void} errorCallback
 * @param {Function} successCallback
 */
Database.prototype.changeVersion = function(
    oldVersion, newVersion, callback, errorCallback, successCallback) {};

/**
 * @interface
 */
function DatabaseCallback() {}

/**
 * @param {!Database} db
 * @return {undefined}
 */
DatabaseCallback.prototype.handleEvent = function(db) {};

/**
 * @constructor
 */
function SQLError() {}

/**
 * @type {number}
 */
SQLError.prototype.code;

/**
 * @type {string}
 */
SQLError.prototype.message;

/**
 * @constructor
 */
function SQLTransaction() {}

/**
 * @param {string} sqlStatement
 * @param {Array.<*>=} opt_queryArgs
 * @param {SQLStatementCallback=} opt_callback
 * @param {(function(!SQLTransaction, !SQLError) : (boolean|void))=}
 *     opt_errorCallback
 */
SQLTransaction.prototype.executeSql = function(
    sqlStatement, opt_queryArgs, opt_callback, opt_errorCallback) {};

/**
 * @typedef {(function(!SQLTransaction, !SQLResultSet) : void)}
 */
var SQLStatementCallback;

/**
 * @constructor
 */
function SQLResultSet() {}

/**
 * @type {number}
 */
SQLResultSet.prototype.insertId;

/**
 * @type {number}
 */
SQLResultSet.prototype.rowsAffected;

/**
 * @type {SQLResultSetRowList}
 */
SQLResultSet.prototype.rows;

/**
 * @constructor
 */
function SQLResultSetRowList() {}

/**
 * @type {number}
 */
SQLResultSetRowList.prototype.length;

/**
 * @param {number} index
 * @return {Object}
 * @nosideeffects
 */
SQLResultSetRowList.prototype.item = function(index) {};

/**
 * @param {string} name
 * @param {string} version
 * @param {string} description
 * @param {number} size
 * @param {(DatabaseCallback|function(Database))=} opt_callback
 * @return {Database}
 */
function openDatabase(name, version, description, size, opt_callback) {}

/**
 * @param {string} name
 * @param {string} version
 * @param {string} description
 * @param {number} size
 * @param {(DatabaseCallback|function(Database))=} opt_callback
 * @return {Database}
 */
Window.prototype.openDatabase =
    function(name, version, description, size, opt_callback) {};

/**
 * @type {boolean}
 */
HTMLImageElement.prototype.complete;

/**
 * @type {string}
 * @see http://www.whatwg.org/specs/web-apps/current-work/multipage/embedded-content-1.html#attr-img-crossorigin
 */
HTMLImageElement.prototype.crossOrigin;

/**
 * This is a superposition of the Window and Worker postMessage methods.
 * @param {*} message
 * @param {(string|!Array.<!Transferable>)=} opt_targetOriginOrTransfer
 * @param {(string|!Array.<!MessagePort>|!Array.<!Transferable>)=}
 *     opt_targetOriginOrPortsOrTransfer
 * @return {void}
 */
function postMessage(message, opt_targetOriginOrTransfer,
    opt_targetOriginOrPortsOrTransfer) {}

/**
 * The postMessage method (as implemented in Opera).
 * @param {string} message
 */
Document.prototype.postMessage = function(message) {};

/**
 * Document head accessor.
 * @see http://www.whatwg.org/specs/web-apps/current-work/multipage/dom.html#the-head-element-0
 * @type {HTMLHeadElement}
 */
Document.prototype.head;

/**
 * @see https://developer.apple.com/webapps/docs/documentation/AppleApplications/Reference/SafariJSRef/DOMApplicationCache/DOMApplicationCache.html
 * @constructor
 * @implements {EventTarget}
 */
function DOMApplicationCache() {}

/**
 * @param {boolean=} opt_useCapture
 * @override
 */
DOMApplicationCache.prototype.addEventListener = function(
    type, listener, opt_useCapture) {};

/**
 * @param {boolean=} opt_useCapture
 * @override
 */
DOMApplicationCache.prototype.removeEventListener = function(
    type, listener, opt_useCapture) {};

/** @override */
DOMApplicationCache.prototype.dispatchEvent = function(evt) {};

/**
 * The object isn't associated with an application cache. This can occur if the
 * update process fails and there is no previous cache to revert to, or if there
 * is no manifest file.
 * @type {number}
 */
DOMApplicationCache.prototype.UNCACHED = 0;

/**
 * The cache is idle.
 * @type {number}
 */
DOMApplicationCache.prototype.IDLE = 1;

/**
 * The update has started but the resources are not downloaded yet - for
 * example, this can happen when the manifest file is fetched.
 * @type {number}
 */
DOMApplicationCache.prototype.CHECKING = 2;

/**
 * The resources are being downloaded into the cache.
 * @type {number}
 */
DOMApplicationCache.prototype.DOWNLOADING = 3;

/**
 * Resources have finished downloading and the new cache is ready to be used.
 * @type {number}
 */
DOMApplicationCache.prototype.UPDATEREADY = 4;

/**
 * The cache is obsolete.
 * @type {number}
 */
DOMApplicationCache.prototype.OBSOLETE = 5;

/**
 * The current status of the application cache.
 * @type {number}
 */
DOMApplicationCache.prototype.status;

/**
 * Sent when the update process finishes for the first time; that is, the first
 * time an application cache is saved.
 * @type {?function(!Event)}
 */
DOMApplicationCache.prototype.oncached;

/**
 * Sent when the cache update process begins.
 * @type {?function(!Event)}
 */
DOMApplicationCache.prototype.onchecking;

/**
 * Sent when the update process begins downloading resources in the manifest
 * file.
 * @type {?function(!Event)}
 */
DOMApplicationCache.prototype.ondownloading;

/**
 * Sent when an error occurs.
 * @type {?function(!Event)}
 */
DOMApplicationCache.prototype.onerror;

/**
 * Sent when the update process finishes but the manifest file does not change.
 * @type {?function(!Event)}
 */
DOMApplicationCache.prototype.onnoupdate;

/**
 * Sent when each resource in the manifest file begins to download.
 * @type {?function(!Event)}
 */
DOMApplicationCache.prototype.onprogress;

/**
 * Sent when there is an existing application cache, the update process
 * finishes, and there is a new application cache ready for use.
 * @type {?function(!Event)}
 */
DOMApplicationCache.prototype.onupdateready;

/**
 * Replaces the active cache with the latest version.
 * @throws {DOMException}
 */
DOMApplicationCache.prototype.swapCache = function() {};

/**
 * Manually triggers the update process.
 * @throws {DOMException}
 */
DOMApplicationCache.prototype.update = function() {};

/** @type {DOMApplicationCache} */
var applicationCache;

/** @type {DOMApplicationCache} */
Window.prototype.applicationCache;

/**
 * @see https://developer.mozilla.org/En/DOM/Worker/Functions_available_to_workers
 * @param {...string} var_args
 */
Window.prototype.importScripts = function(var_args) {};

/**
 * @see https://developer.mozilla.org/En/DOM/Worker/Functions_available_to_workers
 * @param {...string} var_args
 */
var importScripts = function(var_args) {};

/**
 * @see http://dev.w3.org/html5/workers/
 * @constructor
 * @implements {EventTarget}
 */
function WebWorker() {}

/**
 * @param {boolean=} opt_useCapture
 * @override
 */
WebWorker.prototype.addEventListener = function(
    type, listener, opt_useCapture) {};

/**
 * @param {boolean=} opt_useCapture
 * @override
 */
WebWorker.prototype.removeEventListener = function(
    type, listener, opt_useCapture) {};

/** @override */
WebWorker.prototype.dispatchEvent = function(evt) {};

/**
 * Stops the worker process
 */
WebWorker.prototype.terminate = function() {};

/**
 * Posts a message to the worker thread.
 * @param {string} message
 */
WebWorker.prototype.postMessage = function(message) {};

/**
 * Sent when the worker thread posts a message to its creator.
 * @type {?function(!MessageEvent.<*>)}
 */
WebWorker.prototype.onmessage;

/**
 * Sent when the worker thread encounters an error.
 * TODO(tbreisacher): Should this change to function(!ErrorEvent)?
 * @type {?function(!Event)}
 */
WebWorker.prototype.onerror;

/**
 * @see http://dev.w3.org/html5/workers/
 * @constructor
 * @implements {EventTarget}
 */
function Worker(opt_arg0) {}

/**
 * @param {boolean=} opt_useCapture
 * @override
 */
Worker.prototype.addEventListener = function(
    type, listener, opt_useCapture) {};

/**
 * @param {boolean=} opt_useCapture
 * @override
 */
Worker.prototype.removeEventListener = function(
    type, listener, opt_useCapture) {};

/** @override */
Worker.prototype.dispatchEvent = function(evt) {};

/**
 * Stops the worker process
 */
Worker.prototype.terminate = function() {};

/**
 * Posts a message to the worker thread.
 * @param {*} message
 * @param {Array.<!Transferable>=} opt_transfer
 */
Worker.prototype.postMessage = function(message, opt_transfer) {};

/**
 * Posts a message to the worker thread.
 * @param {*} message
 * @param {Array.<!Transferable>=} opt_transfer
 */
Worker.prototype.webkitPostMessage = function(message, opt_transfer) {};

/**
 * Sent when the worker thread posts a message to its creator.
 * @type {?function(!MessageEvent.<*>)}
 */
Worker.prototype.onmessage;

/**
 * Sent when the worker thread encounters an error.
 * TODO(tbreisacher): Should this change to function(!ErrorEvent)?
 * @type {?function(!Event)}
 */
Worker.prototype.onerror;

/**
 * @see http://dev.w3.org/html5/workers/
 * @param {string} scriptURL The URL of the script to run in the SharedWorker.
 * @param {string=} opt_name A name that can later be used to obtain a
 *     reference to the same SharedWorker.
 * @constructor
 * @implements {EventTarget}
 */
function SharedWorker(scriptURL, opt_name) {}

/**
 * @param {boolean=} opt_useCapture
 * @override
 */
SharedWorker.prototype.addEventListener = function(
    type, listener, opt_useCapture) {};

/**
 * @param {boolean=} opt_useCapture
 * @override
 */
SharedWorker.prototype.removeEventListener = function(
    type, listener, opt_useCapture) {};

/** @override */
SharedWorker.prototype.dispatchEvent = function(evt) {};

/**
 * @type {!MessagePort}
 */
SharedWorker.prototype.port;

/**
 * Called on network errors for loading the initial script.
 * TODO(tbreisacher): Should this change to function(!ErrorEvent)?
 * @type {?function(!Event)}
 */
SharedWorker.prototype.onerror;

/**
 * @see http://dev.w3.org/html5/workers/
 * @interface
 */
function WorkerLocation() {}

/** @type {string} */
WorkerLocation.prototype.protocol;

/** @type {string} */
WorkerLocation.prototype.host;

/** @type {string} */
WorkerLocation.prototype.hostname;

/** @type {string} */
WorkerLocation.prototype.port;

/** @type {string} */
WorkerLocation.prototype.pathname;

/** @type {string} */
WorkerLocation.prototype.search;

/** @type {string} */
WorkerLocation.prototype.hash;

/**
 * @see http://dev.w3.org/html5/workers/
 * @interface
 * @extends {EventTarget}
 */
function WorkerGlobalScope() {}

/** @type {WorkerGlobalScope} */
WorkerGlobalScope.prototype.self;

/** @type {WorkerLocation} */
WorkerGlobalScope.prototype.location;

/**
 * Closes the worker represented by this WorkerGlobalScope.
 */
WorkerGlobalScope.prototype.close = function() {};

/**
 * Sent when the worker encounters an error.
 * @type {?function(!Event)}
 */
WorkerGlobalScope.prototype.onerror;

/**
 * Sent when the worker goes offline.
 * @type {?function(!Event)}
 */
WorkerGlobalScope.prototype.onoffline;

/**
 * Sent when the worker goes online.
 * @type {?function(!Event)}
 */
WorkerGlobalScope.prototype.ononline;

/**
 * @see http://dev.w3.org/html5/workers/
 * @interface
 * @extends {WorkerGlobalScope}
 */
function DedicatedWorkerGlobalScope() {}

/**
 * Posts a message to creator of this worker.
 * @param {*} message
 * @param {Array.<!Transferable>=} opt_transfer
 */
DedicatedWorkerGlobalScope.prototype.postMessage =
    function(message, opt_transfer) {};

/**
 * Posts a message to creator of this worker.
 * @param {*} message
 * @param {Array.<!Transferable>=} opt_transfer
 */
DedicatedWorkerGlobalScope.prototype.webkitPostMessage =
    function(message, opt_transfer) {};

/**
 * Sent when the creator posts a message to this worker.
 * @type {?function(!MessageEvent.<*>)}
 */
DedicatedWorkerGlobalScope.prototype.onmessage;

/**
 * @see http://dev.w3.org/html5/workers/
 * @interface
 * @extends {WorkerGlobalScope}
 */
function SharedWorkerGlobalScope() {}

/** @type {string} */
SharedWorkerGlobalScope.prototype.name;

/**
 * Sent when a connection to this worker is opened.
 * @type {?function(!Event)}
 */
SharedWorkerGlobalScope.prototype.onconnect;

/** @type {Element} */
HTMLElement.prototype.contextMenu;

/** @type {boolean} */
HTMLElement.prototype.draggable;

/**
 * This is actually a DOMSettableTokenList property. However since that
 * interface isn't currently defined and no known browsers implement this
 * feature, just define the property for now.
 *
 * @const
 * @type {Object}
 */
HTMLElement.prototype.dropzone;

/**
 * @see http://www.w3.org/TR/html5/dom.html#dom-getelementsbyclassname
 * @param {string} classNames
 * @return {!NodeList}
 * @nosideeffects
 */
HTMLElement.prototype.getElementsByClassName = function(classNames) {};
// NOTE: Document.prototype.getElementsByClassName is in gecko_dom.js

/** @type {boolean} */
HTMLElement.prototype.hidden;

/** @type {boolean} */
HTMLElement.prototype.spellcheck;

/**
 * @see http://www.w3.org/TR/components-intro/
 * @return {!ShadowRoot}
 */
HTMLElement.prototype.createShadowRoot;

/**
 * @see http://www.w3.org/TR/components-intro/
 * @return {!ShadowRoot}
 */
HTMLElement.prototype.webkitCreateShadowRoot;

/**
 * @see http://www.w3.org/TR/shadow-dom/
 * @type {ShadowRoot}
 */
HTMLElement.prototype.shadowRoot;

/**
 * @see http://www.w3.org/TR/shadow-dom/
 * @return {!NodeList}
 */
HTMLElement.prototype.getDestinationInsertionPoints = function() {};

/**
 * @see http://www.w3.org/TR/components-intro/
 * @type {function()}
 */
HTMLElement.prototype.createdCallback;

/**
 * @see http://w3c.github.io/webcomponents/explainer/#lifecycle-callbacks
 * @type {function()}
 */
HTMLElement.prototype.attachedCallback;

/**
 * @see http://w3c.github.io/webcomponents/explainer/#lifecycle-callbacks
 * @type {function()}
 */
HTMLElement.prototype.detachedCallback;

/** @type {string} */
HTMLAnchorElement.prototype.hash;

/** @type {string} */
HTMLAnchorElement.prototype.host;

/** @type {string} */
HTMLAnchorElement.prototype.hostname;

/** @type {string} */
HTMLAnchorElement.prototype.pathname;

/**
 * The 'ping' attribute is known to be supported in recent versions (as of
 * mid-2014) of Chrome, Safari, and Firefox, and is not supported in any
 * current version of Internet Explorer.
 *
 * @type {string}
 * @see http://www.whatwg.org/specs/web-apps/current-work/multipage/semantics.html#hyperlink-auditing
 */
HTMLAnchorElement.prototype.ping;

/** @type {string} */
HTMLAnchorElement.prototype.port;

/** @type {string} */
HTMLAnchorElement.prototype.protocol;

/** @type {string} */
HTMLAnchorElement.prototype.search;

/**
 * @type {string}
 * @see http://www.whatwg.org/specs/web-apps/current-work/multipage/semantics.html#hyperlink-auditing
 */
HTMLAreaElement.prototype.ping;

/**
 * @type {string}
 * @see http://www.w3.org/TR/html-markup/iframe.html#iframe.attrs.srcdoc
 */
HTMLIFrameElement.prototype.srcdoc;

/** @type {string} */
HTMLInputElement.prototype.autocomplete;

/** @type {string} */
HTMLInputElement.prototype.dirname;

/** @type {FileList} */
HTMLInputElement.prototype.files;

/** @type {string} */
HTMLInputElement.prototype.list;

/** @type {string} */
HTMLInputElement.prototype.max;

/** @type {string} */
HTMLInputElement.prototype.min;

/** @type {string} */
HTMLInputElement.prototype.pattern;

/** @type {boolean} */
HTMLInputElement.prototype.multiple;

/** @type {string} */
HTMLInputElement.prototype.placeholder;

/** @type {boolean} */
HTMLInputElement.prototype.required;

/** @type {string} */
HTMLInputElement.prototype.step;

/** @type {Date} */
HTMLInputElement.prototype.valueAsDate;

/** @type {number} */
HTMLInputElement.prototype.valueAsNumber;

/**
 * Changes the form control's value by the value given in the step attribute
 * multiplied by opt_n.
 * @param {number=} opt_n step multiplier.  Defaults to 1.
 */
HTMLInputElement.prototype.stepDown = function(opt_n) {};

/**
 * Changes the form control's value by the value given in the step attribute
 * multiplied by opt_n.
 * @param {number=} opt_n step multiplier.  Defaults to 1.
 */
HTMLInputElement.prototype.stepUp = function(opt_n) {};



/**
 * @constructor
 * @extends {HTMLElement}
 * @see https://developer.mozilla.org/en-US/docs/Web/API/HTMLMediaElement
 */
function HTMLMediaElement() {}

/**
 * @type {number}
 * @const
 */
HTMLMediaElement.HAVE_NOTHING;  // = 0

/**
 * @type {number}
 * @const
 */
HTMLMediaElement.HAVE_METADATA;  // = 1

/**
 * @type {number}
 * @const
 */
HTMLMediaElement.HAVE_CURRENT_DATA;  // = 2

/**
 * @type {number}
 * @const
 */
HTMLMediaElement.HAVE_FUTURE_DATA;  // = 3

/**
 * @type {number}
 * @const
 */
HTMLMediaElement.HAVE_ENOUGH_DATA;  // = 4

/** @type {MediaError} */
HTMLMediaElement.prototype.error;

/** @type {string} */
HTMLMediaElement.prototype.src;

/** @type {string} */
HTMLMediaElement.prototype.currentSrc;

/** @type {number} */
HTMLMediaElement.prototype.networkState;

/** @type {boolean} */
HTMLMediaElement.prototype.autobuffer;

/** @type {TimeRanges} */
HTMLMediaElement.prototype.buffered;

/**
 * Loads the media element.
 */
HTMLMediaElement.prototype.load = function() {};

/**
 * @param {string} type Type of the element in question in question.
 * @return {string} Whether it can play the type.
 * @nosideeffects
 */
HTMLMediaElement.prototype.canPlayType = function(type) {};

/**
 * Callback when the media is buffered and ready to play through.
 * @type {function(!Event)}
 */
HTMLMediaElement.prototype.oncanplaythrough;

/** @type {number} */
HTMLMediaElement.prototype.readyState;

/** @type {boolean} */
HTMLMediaElement.prototype.seeking;

/**
 * The current time, in seconds.
 * @type {number}
 */
HTMLMediaElement.prototype.currentTime;

/**
 * The absolute timeline offset.
 * @return {!Date}
 */
HTMLMediaElement.prototype.getStartDate = function() {};

/**
 * The length of the media in seconds.
 * @type {number}
 */
HTMLMediaElement.prototype.duration;

/** @type {boolean} */
HTMLMediaElement.prototype.paused;

/** @type {number} */
HTMLMediaElement.prototype.defaultPlaybackRate;

/** @type {number} */
HTMLMediaElement.prototype.playbackRate;

/** @type {TimeRanges} */
HTMLMediaElement.prototype.played;

/** @type {TimeRanges} */
HTMLMediaElement.prototype.seekable;

/** @type {boolean} */
HTMLMediaElement.prototype.ended;

/** @type {boolean} */
HTMLMediaElement.prototype.autoplay;

/** @type {boolean} */
HTMLMediaElement.prototype.loop;

/**
 * Starts playing the media.
 */
HTMLMediaElement.prototype.play = function() {};

/**
 * Pauses the media.
 */
HTMLMediaElement.prototype.pause = function() {};

/** @type {boolean} */
HTMLMediaElement.prototype.controls;

/**
 * The audio volume, from 0.0 (silent) to 1.0 (loudest).
 * @type {number}
 */
HTMLMediaElement.prototype.volume;

/** @type {boolean} */
HTMLMediaElement.prototype.muted;

/**
 * @see http://www.whatwg.org/specs/web-apps/current-work/multipage/the-video-element.html#dom-media-addtexttrack
 * @param {string} kind Kind of the text track.
 * @param {string=} opt_label Label of the text track.
 * @param {string=} opt_language Language of the text track.
 * @return {TextTrack} TextTrack object added to the media element.
 */
HTMLMediaElement.prototype.addTextTrack =
    function(kind, opt_label, opt_language) {};

/** @type {TextTrackList} */
HTMLMediaElement.prototype.textTracks;


/**
 * @see http://www.w3.org/TR/shadow-dom/
 * @return {!NodeList}
 */
Text.prototype.getDestinationInsertionPoints = function() {};


/**
 * @see http://www.whatwg.org/specs/web-apps/current-work/multipage/the-video-element.html#texttracklist
 * @constructor
 */
function TextTrackList() {}

/** @type {number} */
TextTrackList.prototype.length;

/**
 * @param {string} id
 * @return {TextTrack}
 */
TextTrackList.prototype.getTrackById = function(id) {};


/**
 * @see http://www.whatwg.org/specs/web-apps/current-work/multipage/the-video-element.html#texttrack
 * @constructor
 * @implements {EventTarget}
 */
function TextTrack() {}

/**
 * @param {TextTrackCue} cue
 */
TextTrack.prototype.addCue = function(cue) {};

/**
 * @param {TextTrackCue} cue
 */
TextTrack.prototype.removeCue = function(cue) {};

/**
 * @const {TextTrackCueList}
 */
TextTrack.prototype.activeCues;

/**
 * @const {TextTrackCueList}
 */
TextTrack.prototype.cues;

/** @override */
TextTrack.prototype.addEventListener = function(type, listener, useCapture) {};

/** @override */
TextTrack.prototype.dispatchEvent = function(evt) {};

/** @override */
TextTrack.prototype.removeEventListener = function(type, listener, useCapture)
    {};



/**
 * @see http://www.whatwg.org/specs/web-apps/current-work/multipage/the-video-element.html#texttrackcuelist
 * @constructor
 */
function TextTrackCueList() {}

/** @const {number} */
TextTrackCueList.prototype.length;

/**
 * @param {string} id
 * @return {TextTrackCue}
 */
TextTrackCueList.prototype.getCueById = function(id) {};



/**
 * @see http://www.whatwg.org/specs/web-apps/current-work/multipage/the-video-element.html#texttrackcue
 * @constructor
 * @param {number} startTime
 * @param {number} endTime
 * @param {string} text
 */
function TextTrackCue(startTime, endTime, text) {}

/** @type {string} */
TextTrackCue.prototype.id;

/** @type {number} */
TextTrackCue.prototype.startTime;

/** @type {number} */
TextTrackCue.prototype.endTime;

/** @type {string} */
TextTrackCue.prototype.text;


/**
 * @see http://dev.w3.org/html5/webvtt/#the-vttcue-interface
 * @constructor
 * @extends {TextTrackCue}
 */
function VTTCue(startTime, endTime, text) {}


/**
 * @constructor
 * @extends {HTMLMediaElement}
 */
function HTMLAudioElement() {}

/**
 * @constructor
 * @extends {HTMLMediaElement}
 * The webkit-prefixed attributes are defined in
 * https://code.google.com/p/chromium/codesearch#chromium/src/third_party/WebKit/Source/core/html/HTMLVideoElement.idl
 */
function HTMLVideoElement() {}

/**
 * Starts displaying the video in full screen mode.
 */
HTMLVideoElement.prototype.webkitEnterFullscreen = function() {};

/**
 * Starts displaying the video in full screen mode.
 */
HTMLVideoElement.prototype.webkitEnterFullScreen = function() {};

/**
 * Stops displaying the video in full screen mode.
 */
HTMLVideoElement.prototype.webkitExitFullscreen = function() {};

/**
 * Stops displaying the video in full screen mode.
 */
HTMLVideoElement.prototype.webkitExitFullScreen = function() {};

/** @type {string} */
HTMLVideoElement.prototype.width;

/** @type {string} */
HTMLVideoElement.prototype.height;

/** @type {number} */
HTMLVideoElement.prototype.videoWidth;

/** @type {number} */
HTMLVideoElement.prototype.videoHeight;

/** @type {string} */
HTMLVideoElement.prototype.poster;

/** @type {boolean} */
HTMLVideoElement.prototype.webkitSupportsFullscreen;

/** @type {boolean} */
HTMLVideoElement.prototype.webkitDisplayingFullscreen;

/** @type {number} */
HTMLVideoElement.prototype.webkitDecodedFrameCount;

/** @type {number} */
HTMLVideoElement.prototype.webkitDroppedFrameCount;

/**
 * @constructor
 */
function MediaError() {}

/** @type {number} */
MediaError.prototype.code;

// HTML5 MessageChannel
/**
 * @see http://dev.w3.org/html5/spec/comms.html#messagechannel
 * @constructor
 */
function MessageChannel() {}

/**
 * Returns the first port.
 * @type {!MessagePort}
 */
MessageChannel.prototype.port1;

/**
 * Returns the second port.
 * @type {!MessagePort}
 */
MessageChannel.prototype.port2;

// HTML5 MessagePort
/**
 * @see http://dev.w3.org/html5/spec/comms.html#messageport
 * @constructor
 * @implements {EventTarget}
 * @implements {Transferable}
 */
function MessagePort() {}

/**
 * @param {boolean=} opt_useCapture
 * @override
 */
MessagePort.prototype.addEventListener = function(
    type, listener, opt_useCapture) {};

/**
 * @param {boolean=} opt_useCapture
 * @override
 */
MessagePort.prototype.removeEventListener = function(
    type, listener, opt_useCapture) {};

/** @override */
MessagePort.prototype.dispatchEvent = function(evt) {};


/**
 * Posts a message through the channel, optionally with the given
 * Array of Transferables.
 * @param {*} message
 * @param {Array.<!Transferable>=} opt_transfer
 */
MessagePort.prototype.postMessage = function(message, opt_transfer) {
};

/**
 * Begins dispatching messages received on the port.
 */
MessagePort.prototype.start = function() {};

/**
 * Disconnects the port, so that it is no longer active.
 */
MessagePort.prototype.close = function() {};

/**
 * TODO(blickly): Change this to MessageEvent.<*> and add casts as needed
 * @type {?function(!MessageEvent.<?>)}
 */
MessagePort.prototype.onmessage;

// HTML5 MessageEvent class
/**
 * @see http://dev.w3.org/html5/spec/comms.html#messageevent
 * @constructor
 * @extends {Event}
 * @template T
 */
function MessageEvent() {}

/**
 * The data payload of the message.
 * @type {T}
 */
MessageEvent.prototype.data;

/**
 * The origin of the message, for server-sent events and cross-document
 * messaging.
 * @type {string}
 */
MessageEvent.prototype.origin;

/**
 * The last event ID, for server-sent events.
 * @type {string}
 */
MessageEvent.prototype.lastEventId;

/**
 * The window that dispatched the event.
 * @type {Window}
 */
MessageEvent.prototype.source;

/**
 * The Array of MessagePorts sent with the message, for cross-document
 * messaging and channel messaging.
 * @type {Array.<MessagePort>}
 */
MessageEvent.prototype.ports;

/**
 * Initializes the event in a manner analogous to the similarly-named methods in
 * the DOM Events interfaces.
 * @param {string} typeArg
 * @param {boolean} canBubbleArg
 * @param {boolean} cancelableArg
 * @param {T} dataArg
 * @param {string} originArg
 * @param {string} lastEventIdArg
 * @param {Window} sourceArg
 * @param {Array.<MessagePort>} portsArg
 */
MessageEvent.prototype.initMessageEvent = function(typeArg, canBubbleArg,
    cancelableArg, dataArg, originArg, lastEventIdArg, sourceArg, portsArg) {};

/**
 * Initializes the event in a manner analogous to the similarly-named methods in
 * the DOM Events interfaces.
 * @param {string} namespaceURI
 * @param {string} typeArg
 * @param {boolean} canBubbleArg
 * @param {boolean} cancelableArg
 * @param {T} dataArg
 * @param {string} originArg
 * @param {string} lastEventIdArg
 * @param {Window} sourceArg
 * @param {Array.<MessagePort>} portsArg
 */
MessageEvent.prototype.initMessageEventNS = function(namespaceURI, typeArg,
    canBubbleArg, cancelableArg, dataArg, originArg, lastEventIdArg, sourceArg,
    portsArg) {};

/**
 * HTML5 DataTransfer class.
 *
 * We say that this extends ClipboardData, because Event.prototype.clipboardData
 * is a DataTransfer on WebKit but a ClipboardData on IE. The interfaces are so
 * similar that it's easier to merge them.
 *
 * @see http://www.w3.org/TR/2011/WD-html5-20110113/dnd.html
 * @see http://www.whatwg.org/specs/web-apps/current-work/multipage/dnd.html
 * @see http://developers.whatwg.org/dnd.html#datatransferitem
 * @constructor
 * @extends {ClipboardData}
 */
function DataTransfer() {}

/** @type {string} */
DataTransfer.prototype.dropEffect;

/** @type {string} */
DataTransfer.prototype.effectAllowed;

/** @type {Array.<string>} */
DataTransfer.prototype.types;

/** @type {FileList} */
DataTransfer.prototype.files;

/**
 * @param {string=} opt_format Format for which to remove data.
 * @override
 */
DataTransfer.prototype.clearData = function(opt_format) {};

/**
 * @param {string} format Format for which to set data.
 * @param {string} data Data to add.
 * @override
 */
DataTransfer.prototype.setData = function(format, data) {};

/**
 * @param {string} format Format for which to set data.
 * @return {string} Data for the given format.
 * @override
 */
DataTransfer.prototype.getData = function(format) { return ''; };

/**
 * @param {HTMLElement} img The image to use when dragging.
 * @param {number} x Horizontal position of the cursor.
 * @param {number} y Vertical position of the cursor.
 */
DataTransfer.prototype.setDragImage = function(img, x, y) {};

/**
 * @param {HTMLElement} elem Element to receive drag result events.
 */
DataTransfer.prototype.addElement = function(elem) {};

/**
 * Addition for accessing clipboard file data that are part of the proposed
 * HTML5 spec.
 * @type {DataTransfer}
 */
MouseEvent.prototype.dataTransfer;

/**
 * @typedef {{
 *   bubbles: (boolean|undefined),
 *   cancelable: (boolean|undefined),
 *   view: (Window|undefined),
 *   detail: (number|undefined),
 *   screenX: (number|undefined),
 *   screenY: (number|undefined),
 *   clientX: (number|undefined),
 *   clientY: (number|undefined),
 *   ctrlKey: (boolean|undefined),
 *   shiftKey: (boolean|undefined),
 *   altKey: (boolean|undefined),
 *   metaKey: (boolean|undefined),
 *   button: (number|undefined),
 *   buttons: (number|undefined),
 *   relatedTarget: (EventTarget|undefined),
 *   deltaX: (number|undefined),
 *   deltaY: (number|undefined),
 *   deltaZ: (number|undefined),
 *   deltaMode: (number|undefined)
 * }}
 */
var WheelEventInit;

/**
 * @param {string} type
 * @param {WheelEventInit=} opt_eventInitDict
 * @see http://www.w3.org/TR/DOM-Level-3-Events/#interface-WheelEvent
 * @constructor
 * @extends {MouseEvent}
 */
var WheelEvent = function(type, opt_eventInitDict) {};

/** @const {number} */
WheelEvent.prototype.deltaX;

/** @const {number} */
WheelEvent.prototype.deltaY;

/** @const {number} */
WheelEvent.prototype.deltaZ;

/** @const {number} */
WheelEvent.prototype.deltaMode;

/**
 * HTML5 DataTransferItem class.
 *
 * @see http://www.w3.org/TR/2011/WD-html5-20110113/dnd.html
 * @see http://www.whatwg.org/specs/web-apps/current-work/multipage/dnd.html
 * @see http://developers.whatwg.org/dnd.html#datatransferitem
 * @constructor
 */
var DataTransferItem = function() {};

/** @type {string} */
DataTransferItem.prototype.kind;

/** @type {string} */
DataTransferItem.prototype.type;

/**
 * @param {function(string)} callback
 * @nosideeffects
 */
DataTransferItem.prototype.getAsString = function(callback) {};

/**
 * @return {?File} The file corresponding to this item, or null.
 * @nosideeffects
 */
DataTransferItem.prototype.getAsFile = function() { return null; };

/**
 * @return {?Entry} The Entry corresponding to this item, or null. Note that
 * despite its name,this method only works in Chrome, and will eventually
 * be renamed to {@code getAsEntry}.
 * @nosideeffects
 */
DataTransferItem.prototype.webkitGetAsEntry = function() { return null; };

/**
 * HTML5 DataTransferItemList class. There are some discrepancies in the docs
 * on the whatwg.org site. When in doubt, these prototypes match what is
 * implemented as of Chrome 30.
 *
 * @see http://www.w3.org/TR/2011/WD-html5-20110113/dnd.html
 * @see http://www.whatwg.org/specs/web-apps/current-work/multipage/dnd.html
 * @see http://developers.whatwg.org/dnd.html#datatransferitem
 * @constructor
 */
var DataTransferItemList = function() {};

/** @type {number} */
DataTransferItemList.prototype.length;

/**
 * @param {number} i File to return from the list.
 * @return {DataTransferItem} The ith DataTransferItem in the list, or null.
 * @nosideeffects
 */
DataTransferItemList.prototype.item = function(i) { return null; };

/**
 * Adds an item to the list.
 * @param {string|!File} data Data for the item being added.
 * @param {string=} opt_type Mime type of the item being added. MUST be present
 *     if the {@code data} parameter is a string.
 */
DataTransferItemList.prototype.add = function(data, opt_type) {};

/**
 * Removes an item from the list.
 * @param {number} i File to remove from the list.
 */
DataTransferItemList.prototype.remove = function(i) {};

/**
 * Removes all items from the list.
 */
DataTransferItemList.prototype.clear = function() {};

/** @type {!DataTransferItemList} */
DataTransfer.prototype.items;


/**
 * @see http://www.whatwg.org/specs/web-apps/current-work/multipage/dnd.html#the-dragevent-interface
 * @constructor
 * @extends {MouseEvent}
 */
function DragEvent() {}

/** @type {DataTransfer} */
DragEvent.prototype.dataTransfer;


/**
 * @typedef {{
 *   lengthComputable: (boolean|undefined),
 *   loaded: (number|undefined),
 *   total: (number|undefined)
 * }}
 */
var ProgressEventInit;

/**
 * @constructor
 * @param {string} type
 * @param {ProgressEventInit=} opt_progressEventInitDict
 * @extends {Event}
 * @see https://developer.mozilla.org/en-US/docs/Web/API/ProgressEvent
 */
function ProgressEvent(type, opt_progressEventInitDict) {}

/** @type {number} */
ProgressEvent.prototype.total;

/** @type {number} */
ProgressEvent.prototype.loaded;

/** @type {boolean} */
ProgressEvent.prototype.lengthComputable;


/**
 * @constructor
 */
function TimeRanges() {}

/** @type {number} */
TimeRanges.prototype.length;

/**
 * @param {number} index The index.
 * @return {number} The start time of the range at index.
 * @throws {DOMException}
 */
TimeRanges.prototype.start = function(index) { return 0; };

/**
 * @param {number} index The index.
 * @return {number} The end time of the range at index.
 * @throws {DOMException}
 */
TimeRanges.prototype.end = function(index) { return 0; };


// HTML5 Web Socket class
/**
 * @see http://dev.w3.org/html5/websockets/
 * @constructor
 * @param {string} url
 * @param {string=} opt_protocol
 * @implements {EventTarget}
 */
function WebSocket(url, opt_protocol) {}

/**
 * @param {boolean=} opt_useCapture
 * @override
 */
WebSocket.prototype.addEventListener = function(
    type, listener, opt_useCapture) {};

/**
 * @param {boolean=} opt_useCapture
 * @override
 */
WebSocket.prototype.removeEventListener = function(
    type, listener, opt_useCapture) {};

/** @override */
WebSocket.prototype.dispatchEvent = function(evt) {};

/**
 * Returns the URL value that was passed to the constructor.
 * @type {string}
 */
WebSocket.prototype.URL;

/**
 * The connection has not yet been established.
 * @type {number}
 */
WebSocket.prototype.CONNECTING = 0;

/**
 * The Web Socket connection is established and communication is possible.
 * @type {number}
 */
WebSocket.prototype.OPEN = 1;

/**
 * The connection has been closed or could not be opened.
 * @type {number}
 */
WebSocket.prototype.CLOSED = 2;

/**
 * Represents the state of the connection.
 * @type {number}
 */
WebSocket.prototype.readyState;

/**
 * Returns the number of bytes that have been queued but not yet sent.
 * @type {number}
 */
WebSocket.prototype.bufferedAmount;

/**
 * An event handler called on open event.
 * @type {?function(!Event)}
 */
WebSocket.prototype.onopen;

/**
 * An event handler called on message event.
 * TODO(blickly): Change this to MessageEvent.<*> and add casts as needed
 * @type {?function(!MessageEvent.<?>)}
 */
WebSocket.prototype.onmessage;

/**
 * An event handler called on close event.
 * @type {?function(!Event)}
 */
WebSocket.prototype.onclose;

/**
 * Transmits data using the connection.
 * @param {string|ArrayBuffer|ArrayBufferView} data
 * @return {boolean}
 */
WebSocket.prototype.send = function(data) {};

/**
 * Closes the Web Socket connection or connection attempt, if any.
 */
WebSocket.prototype.close = function() {};

/**
 * @type {string} Sets the type of data (blob or arraybuffer) for binary data.
 */
WebSocket.prototype.binaryType;

// HTML5 History
/**
 * Pushes a new state into the session history.
 * @see http://www.w3.org/TR/html5/history.html#the-history-interface
 * @param {*} data New state.
 * @param {string} title The title for a new session history entry.
 * @param {string=} opt_url The URL for a new session history entry.
 */
History.prototype.pushState = function(data, title, opt_url) {};

/**
 * Replaces the current state in the session history.
 * @see http://www.w3.org/TR/html5/history.html#the-history-interface
 * @param {*} data New state.
 * @param {string} title The title for a session history entry.
 * @param {string=} opt_url The URL for a new session history entry.
 */
History.prototype.replaceState = function(data, title, opt_url) {};

/**
 * Pending state object.
 * @see https://developer.mozilla.org/en-US/docs/Web/Guide/API/DOM/Manipulating_the_browser_history#Reading_the_current_state
 * @type {*}
 */
History.prototype.state;

/**
 * @see http://www.whatwg.org/specs/web-apps/current-work/#popstateevent
 * @constructor
 * @extends {Event}
 *
 * @param {string} type
 * @param {{state: *}=} opt_eventInitDict
 */
function PopStateEvent(type, opt_eventInitDict) {}

/**
 * @type {*}
 */
PopStateEvent.prototype.state;

/**
 * Initializes the event after it has been created with document.createEvent
 * @param {string} typeArg
 * @param {boolean} canBubbleArg
 * @param {boolean} cancelableArg
 * @param {*} stateArg
 */
PopStateEvent.prototype.initPopStateEvent = function(typeArg, canBubbleArg,
    cancelableArg, stateArg) {};

/**
 * @see http://www.whatwg.org/specs/web-apps/current-work/#hashchangeevent
 * @constructor
 * @extends {Event}
 *
 * @param {string} type
 * @param {{oldURL: string, newURL: string}=} opt_eventInitDict
 */
function HashChangeEvent(type, opt_eventInitDict) {}

/** @type {string} */
HashChangeEvent.prototype.oldURL;

/** @type {string} */
HashChangeEvent.prototype.newURL;

/**
 * Initializes the event after it has been created with document.createEvent
 * @param {string} typeArg
 * @param {boolean} canBubbleArg
 * @param {boolean} cancelableArg
 * @param {string} oldURLArg
 * @param {string} newURLArg
 */
HashChangeEvent.prototype.initHashChangeEvent = function(typeArg, canBubbleArg,
    cancelableArg, oldURLArg, newURLArg) {};

/**
 * @see http://www.whatwg.org/specs/web-apps/current-work/#pagetransitionevent
 * @constructor
 * @extends {Event}
 *
 * @param {string} type
 * @param {{persisted: boolean}=} opt_eventInitDict
 */
function PageTransitionEvent(type, opt_eventInitDict) {}

/** @type {boolean} */
PageTransitionEvent.prototype.persisted;

/**
 * Initializes the event after it has been created with document.createEvent
 * @param {string} typeArg
 * @param {boolean} canBubbleArg
 * @param {boolean} cancelableArg
 * @param {*} persistedArg
 */
PageTransitionEvent.prototype.initPageTransitionEvent = function(typeArg,
    canBubbleArg, cancelableArg, persistedArg) {};

/**
 * @constructor
 */
function FileList() {}

/** @type {number} */
FileList.prototype.length;

/**
 * @param {number} i File to return from the list.
 * @return {File} The ith file in the list.
 * @nosideeffects
 */
FileList.prototype.item = function(i) { return null; };

/**
 * @type {boolean}
 * @see http://dev.w3.org/2006/webapi/XMLHttpRequest-2/#withcredentials
 */
XMLHttpRequest.prototype.withCredentials;

/**
 * @type {XMLHttpRequestUpload}
 * @see http://dev.w3.org/2006/webapi/XMLHttpRequest-2/#the-upload-attribute
 */
XMLHttpRequest.prototype.upload;

/**
 * @param {string} mimeType The mime type to override with.
 */
XMLHttpRequest.prototype.overrideMimeType = function(mimeType) {};

/**
 * @type {string}
 * @see http://dev.w3.org/2006/webapi/XMLHttpRequest-2/#the-responsetype-attribute
 */
XMLHttpRequest.prototype.responseType;

/**
 * @type {*}
 * @see http://dev.w3.org/2006/webapi/XMLHttpRequest-2/#the-responsetype-attribute
 */
XMLHttpRequest.prototype.response;


/**
 * @type {ArrayBuffer}
 * Implemented as a draft spec in Firefox 4 as the way to get a requested array
 * buffer from an XMLHttpRequest.
 * @see https://developer.mozilla.org/En/Using_XMLHttpRequest#Receiving_binary_data_using_JavaScript_typed_arrays
 */
XMLHttpRequest.prototype.mozResponseArrayBuffer;

/**
 * XMLHttpRequestEventTarget defines events for checking the status of a data
 * transfer between a client and a server. This should be a common base class
 * for XMLHttpRequest and XMLHttpRequestUpload.
 *
 * @constructor
 * @implements {EventTarget}
 */
function XMLHttpRequestEventTarget() {}

/**
 * @param {boolean=} opt_useCapture
 * @override
 */
XMLHttpRequestEventTarget.prototype.addEventListener = function(
    type, listener, opt_useCapture) {};

/**
 * @param {boolean=} opt_useCapture
 * @override
 */
XMLHttpRequestEventTarget.prototype.removeEventListener = function(
    type, listener, opt_useCapture) {};

/** @override */
XMLHttpRequestEventTarget.prototype.dispatchEvent = function(evt) {};

/**
 * An event target to track the status of an upload.
 *
 * @constructor
 * @extends {XMLHttpRequestEventTarget}
 */
function XMLHttpRequestUpload() {}

/**
 * @param {number=} opt_width
 * @param {number=} opt_height
 * @constructor
 * @extends {HTMLImageElement}
 */
function Image(opt_width, opt_height) {}


/**
 * Dataset collection.
 * This is really a DOMStringMap but it behaves close enough to an object to
 * pass as an object.
 * @type {Object}
 * @const
 */
HTMLElement.prototype.dataset;


/**
 * @constructor
 * @see https://dom.spec.whatwg.org/#interface-domtokenlist
 */
function DOMTokenList() {}

/**
 * Returns the number of CSS classes applied to this Element.
 * @type {number}
 */
DOMTokenList.prototype.length;

/**
 * @param {number} index The index of the item to return.
 * @return {string} The CSS class at the specified index.
 * @nosideeffects
 */
DOMTokenList.prototype.item = function(index) {};

/**
 * @param {string} token The CSS class to check for.
 * @return {boolean} Whether the CSS class has been applied to the Element.
 * @nosideeffects
 */
DOMTokenList.prototype.contains = function(token) {};

/**
 * @param {...string} var_args The CSS class(es) to add to this element.
 */
DOMTokenList.prototype.add = function(var_args) {};

/**
 * @param {...string} var_args The CSS class(es) to remove from this element.
 */
DOMTokenList.prototype.remove = function(var_args) {};

/**
 * @param {string} token The CSS class to toggle from this element.
 * @param {boolean=} opt_force True to add the class whether it exists
 *     or not. False to remove the class whether it exists or not.
 *     This argument is not supported on IE 10 and below, according to
 *     the MDN page linked below.
 * @return {boolean} False if the token was removed; True otherwise.
 * @see https://developer.mozilla.org/en-US/docs/Web/API/Element.classList
 */
DOMTokenList.prototype.toggle = function(token, opt_force) {};

/**
 * @return {string} A stringified representation of CSS classes.
 * @nosideeffects
 * @override
 */
DOMTokenList.prototype.toString = function() {};

/**
 * A better interface to CSS classes than className.
 * @type {DOMTokenList}
 * @see http://www.w3.org/TR/html5/elements.html#dom-classlist
 * @const
 */
HTMLElement.prototype.classList;

/**
 * Web Cryptography API
 * @see http://www.w3.org/TR/WebCryptoAPI/
 */

/** @see https://developer.mozilla.org/en/DOM/window.crypto */
Window.prototype.crypto;

/**
 * @see https://developer.mozilla.org/en/DOM/window.crypto.getRandomValues
 * @param {!ArrayBufferView} typedArray
 * @return {!ArrayBufferView}
 * @throws {Error}
 */
Window.prototype.crypto.getRandomValues = function(typedArray) {};

/**
 * Constraint Validation API properties and methods
 * @see http://www.w3.org/TR/2009/WD-html5-20090423/forms.html#the-constraint-validation-api
 */

/** @return {boolean} */
HTMLFormElement.prototype.checkValidity = function() {};

/** @type {boolean} */
HTMLFormElement.prototype.noValidate;

/** @constructor */
function ValidityState() {}

/** @type {boolean} */
ValidityState.prototype.customError;

/** @type {boolean} */
ValidityState.prototype.patternMismatch;

/** @type {boolean} */
ValidityState.prototype.rangeOverflow;

/** @type {boolean} */
ValidityState.prototype.rangeUnderflow;

/** @type {boolean} */
ValidityState.prototype.stepMismatch;

/** @type {boolean} */
ValidityState.prototype.typeMismatch;

/** @type {boolean} */
ValidityState.prototype.tooLong;

/** @type {boolean} */
ValidityState.prototype.valid;

/** @type {boolean} */
ValidityState.prototype.valueMissing;


/** @type {boolean} */
HTMLButtonElement.prototype.autofocus;

/**
 * @const
 * @type {NodeList}
 */
HTMLButtonElement.prototype.labels;

/** @type {string} */
HTMLButtonElement.prototype.validationMessage;

/**
 * @const
 * @type {ValidityState}
 */
HTMLButtonElement.prototype.validity;

/** @type {boolean} */
HTMLButtonElement.prototype.willValidate;

/** @return {boolean} */
HTMLButtonElement.prototype.checkValidity = function() {};

/** @param {string} message */
HTMLButtonElement.prototype.setCustomValidity = function(message) {};

/**
 * @type {string}
 * @see http://www.w3.org/TR/html5/forms.html#attr-fs-formaction
 */
HTMLButtonElement.prototype.formAction;

/**
 * @type {string}
 * @see http://www.w3.org/TR/html5/forms.html#attr-fs-formenctype
 */
HTMLButtonElement.prototype.formEnctype;

/**
 * @type {string}
 * @see http://www.w3.org/TR/html5/forms.html#attr-fs-formmethod
 */
HTMLButtonElement.prototype.formMethod;

/**
 * @type {string}
 * @see http://www.w3.org/TR/html5/forms.html#attr-fs-formtarget
 */
HTMLButtonElement.prototype.formTarget;

/** @type {boolean} */
HTMLInputElement.prototype.autofocus;

/** @type {boolean} */
HTMLInputElement.prototype.formNoValidate;

/**
 * @type {string}
 * @see http://www.w3.org/TR/html5/forms.html#attr-fs-formaction
 */
HTMLInputElement.prototype.formAction;

/**
 * @type {string}
 * @see http://www.w3.org/TR/html5/forms.html#attr-fs-formenctype
 */
HTMLInputElement.prototype.formEnctype;

/**
 * @type {string}
 * @see http://www.w3.org/TR/html5/forms.html#attr-fs-formmethod
 */
HTMLInputElement.prototype.formMethod;

/**
 * @type {string}
 * @see http://www.w3.org/TR/html5/forms.html#attr-fs-formtarget
 */
HTMLInputElement.prototype.formTarget;

/**
 * @const
 * @type {NodeList}
 */
HTMLInputElement.prototype.labels;

/** @type {string} */
HTMLInputElement.prototype.validationMessage;

/**
 * @const
 * @type {ValidityState}
 */
HTMLInputElement.prototype.validity;

/** @type {boolean} */
HTMLInputElement.prototype.willValidate;

/** @return {boolean} */
HTMLInputElement.prototype.checkValidity = function() {};

/** @param {string} message */
HTMLInputElement.prototype.setCustomValidity = function(message) {};

/** @type {Element} */
HTMLLabelElement.prototype.control;

/** @type {boolean} */
HTMLSelectElement.prototype.autofocus;

/**
 * @const
 * @type {NodeList}
 */
HTMLSelectElement.prototype.labels;

/** @type {HTMLCollection} */
HTMLSelectElement.prototype.selectedOptions;

/** @type {string} */
HTMLSelectElement.prototype.validationMessage;

/**
 * @const
 * @type {ValidityState}
 */
HTMLSelectElement.prototype.validity;

/** @type {boolean} */
HTMLSelectElement.prototype.willValidate;

/** @return {boolean} */
HTMLSelectElement.prototype.checkValidity = function() {};

/** @param {string} message */
HTMLSelectElement.prototype.setCustomValidity = function(message) {};

/** @type {boolean} */
HTMLTextAreaElement.prototype.autofocus;

/**
 * @const
 * @type {NodeList}
 */
HTMLTextAreaElement.prototype.labels;

/** @type {string} */
HTMLTextAreaElement.prototype.validationMessage;

/**
 * @const
 * @type {ValidityState}
 */
HTMLTextAreaElement.prototype.validity;

/** @type {boolean} */
HTMLTextAreaElement.prototype.willValidate;

/** @return {boolean} */
HTMLTextAreaElement.prototype.checkValidity = function() {};

/** @param {string} message */
HTMLTextAreaElement.prototype.setCustomValidity = function(message) {};

/**
 * @constructor
 * @extends {HTMLElement}
 * @see http://www.w3.org/TR/html5/the-embed-element.html#htmlembedelement
 */
function HTMLEmbedElement() {}

/**
 * @type {string}
 * @see http://www.w3.org/TR/html5/dimension-attributes.html#dom-dim-width
 */
HTMLEmbedElement.prototype.width;

/**
 * @type {string}
 * @see http://www.w3.org/TR/html5/dimension-attributes.html#dom-dim-height
 */
HTMLEmbedElement.prototype.height;

/**
 * @type {string}
 * @see http://www.w3.org/TR/html5/the-embed-element.html#dom-embed-src
 */
HTMLEmbedElement.prototype.src;

/**
 * @type {string}
 * @see http://www.w3.org/TR/html5/the-embed-element.html#dom-embed-type
 */
HTMLEmbedElement.prototype.type;

// Fullscreen APIs.

/**
 * @see http://www.w3.org/TR/2012/WD-fullscreen-20120703/#dom-element-requestfullscreen
 */
Element.prototype.requestFullscreen = function() {};

/**
 * @type {boolean}
 * @see http://www.w3.org/TR/2012/WD-fullscreen-20120703/#dom-document-fullscreenenabled
 */
Document.prototype.fullscreenEnabled;

/**
 * @type {Element}
 * @see http://www.w3.org/TR/2012/WD-fullscreen-20120703/#dom-document-fullscreenelement
 */
Document.prototype.fullscreenElement;

/**
 * @see http://www.w3.org/TR/2012/WD-fullscreen-20120703/#dom-document-exitfullscreen
 */
Document.prototype.exitFullscreen = function() {};

// Externs definitions of browser current implementations.
// Firefox 10 implementation.
Element.prototype.mozRequestFullScreen = function() {};

Element.prototype.mozRequestFullScreenWithKeys = function() {};

/** @type {boolean} */
Document.prototype.mozFullScreen;

Document.prototype.mozCancelFullScreen = function() {};

/** @type {Element} */
Document.prototype.mozFullScreenElement;

/** @type {boolean} */
Document.prototype.mozFullScreenEnabled;

// Chrome 21 implementation.
/**
 * The current fullscreen element for the document is set to this element.
 * Valid only for Webkit browsers.
 * @param {number=} opt_allowKeyboardInput Whether keyboard input is desired.
 *     Should use ALLOW_KEYBOARD_INPUT constant.
 */
Element.prototype.webkitRequestFullScreen = function(opt_allowKeyboardInput) {};

/**
 * The current fullscreen element for the document is set to this element.
 * Valid only for Webkit browsers.
 * @param {number=} opt_allowKeyboardInput Whether keyboard input is desired.
 *     Should use ALLOW_KEYBOARD_INPUT constant.
 */
Element.prototype.webkitRequestFullscreen = function(opt_allowKeyboardInput) {};

/** @type {boolean} */
Document.prototype.webkitIsFullScreen;

Document.prototype.webkitCancelFullScreen = function() {};

/** @type {Element} */
Document.prototype.webkitCurrentFullScreenElement;

/** @type {Element} */
Document.prototype.webkitFullscreenElement;

/** @type {boolean} */
Document.prototype.webkitFullScreenKeyboardInputAllowed;

// IE 11 implementation.
// http://msdn.microsoft.com/en-us/library/ie/dn265028(v=vs.85).aspx
/** @return {void} */
Element.prototype.msRequestFullscreen = function() {};

/** @return {void} */
Element.prototype.msExitFullscreen = function() {};

/** @type {boolean} */
Document.prototype.msFullscreenEnabled;

/** @type {Element} */
Document.prototype.msFullscreenElement;

/** @type {number} */
Element.ALLOW_KEYBOARD_INPUT = 1;

/** @type {number} */
Element.prototype.ALLOW_KEYBOARD_INPUT = 1;


/** @constructor */
function MutationObserverInit() {}

/** @type {boolean} */
MutationObserverInit.prototype.childList;

/** @type {boolean} */
MutationObserverInit.prototype.attributes;

/** @type {boolean} */
MutationObserverInit.prototype.characterData;

/** @type {boolean} */
MutationObserverInit.prototype.subtree;

/** @type {boolean} */
MutationObserverInit.prototype.attributeOldValue;

/** @type {boolean} */
MutationObserverInit.prototype.characterDataOldValue;

/** @type {Array.<string>} */
MutationObserverInit.prototype.attributeFilter;


/** @constructor */
function MutationRecord() {}

/** @type {string} */
MutationRecord.prototype.type;

/** @type {Node} */
MutationRecord.prototype.target;

/** @type {NodeList} */
MutationRecord.prototype.addedNodes;

/** @type {NodeList} */
MutationRecord.prototype.removedNodes;

/** @type {Node} */
MutationRecord.prototype.previouSibling;

/** @type {Node} */
MutationRecord.prototype.nextSibling;

/** @type {?string} */
MutationRecord.prototype.attributeName;

/** @type {?string} */
MutationRecord.prototype.attributeNamespace;

/** @type {?string} */
MutationRecord.prototype.oldValue;


/**
 * @see http://www.w3.org/TR/domcore/#mutation-observers
 * @param {function(Array.<MutationRecord>, MutationObserver)} callback
 * @constructor
 */
function MutationObserver(callback) {}

/**
 * @param {Node} target
 * @param {MutationObserverInit=} options
 */
MutationObserver.prototype.observe = function(target, options) {};

MutationObserver.prototype.disconnect = function() {};

/**
 * @type {function(new:MutationObserver, function(Array.<MutationRecord>))}
 */
Window.prototype.WebKitMutationObserver;

/**
 * @type {function(new:MutationObserver, function(Array.<MutationRecord>))}
 */
Window.prototype.MozMutationObserver;


/**
 * @see http://www.w3.org/TR/page-visibility/
 * @type {VisibilityState}
 */
Document.prototype.visibilityState;

/**
 * @type {string}
 */
Document.prototype.webkitVisibilityState;

/**
 * @type {string}
 */
Document.prototype.msVisibilityState;

/**
 * @see http://www.w3.org/TR/page-visibility/
 * @type {boolean}
 */
Document.prototype.hidden;

/**
 * @type {boolean}
 */
Document.prototype.webkitHidden;

/**
 * @type {boolean}
 */
Document.prototype.msHidden;

/**
 * @see http://www.w3.org/TR/components-intro/
 * @see http://w3c.github.io/webcomponents/spec/custom/#extensions-to-document-interface-to-register
 * @param {string} type
 * @param {{extends: (string|undefined), prototype: (Object|undefined)}} options
 */
Document.prototype.registerElement;

/**
 * This method is deprecated and should be removed by the end of 2014.
 * @see http://www.w3.org/TR/components-intro/
 * @see http://w3c.github.io/webcomponents/spec/custom/#extensions-to-document-interface-to-register
 * @param {string} type
 * @param {{extends: (string|undefined), prototype: (Object|undefined)}} options
 */
Document.prototype.register;

/**
 * @type {!FontFaceSet}
 * @see http://dev.w3.org/csswg/css-font-loading/#dom-fontfacesource-fonts
 */
Document.prototype.fonts;


/**
 * Definition of ShadowRoot interface,
 * @see http://www.w3.org/TR/shadow-dom/#api-shadow-root
 * @constructor
 * @extends {DocumentFragment}
 */
function ShadowRoot() {}

/**
 * The host element that a ShadowRoot is attached to.
 * Note: this is not yet W3C standard but is undergoing development.
 * W3C feature tracking bug:
 * https://www.w3.org/Bugs/Public/show_bug.cgi?id=22399
 * Draft specification:
 * https://dvcs.w3.org/hg/webcomponents/raw-file/6743f1ace623/spec/shadow/index.html#shadow-root-object
 * @type {!Element}
 */
ShadowRoot.prototype.host;

/**
 * @param {string} id id.
 * @return {HTMLElement}
 * @nosideeffects
 */
ShadowRoot.prototype.getElementById = function(id) {};


/**
 * @param {string} className
 * @return {!NodeList}
 * @nosideeffects
 */
ShadowRoot.prototype.getElementsByClassName = function(className) {};


/**
 * @param {string} tagName
 * @return {!NodeList}
 * @nosideeffects
 */
ShadowRoot.prototype.getElementsByTagName = function(tagName) {};


/**
 * @param {string} namespace
 * @param {string} localName
 * @return {!NodeList}
 * @nosideeffects
 */
ShadowRoot.prototype.getElementsByTagNameNS = function(namespace, localName) {};


/**
 * @return {Selection}
 * @nosideeffects
 */
ShadowRoot.prototype.getSelection = function() {};


/**
 * @param {number} x
 * @param {number} y
 * @return {Element}
 * @nosideeffects
 */
ShadowRoot.prototype.elementFromPoint = function(x, y) {};


/**
 * @type {boolean}
 */
ShadowRoot.prototype.applyAuthorStyles;


/**
 * @type {boolean}
 */
ShadowRoot.prototype.resetStyleInheritance;


/**
 * @type {Element}
 */
ShadowRoot.prototype.activeElement;


/**
 * @type {?ShadowRoot}
 */
ShadowRoot.prototype.olderShadowRoot;


/**
 * @type {string}
 */
ShadowRoot.prototype.innerHTML;


/**
 * @type {!StyleSheetList}
 */
ShadowRoot.prototype.styleSheets;



/**
 * @see http://www.w3.org/TR/shadow-dom/#the-content-element
 * @constructor
 * @extends {HTMLElement}
 */
function HTMLContentElement() {}

/**
 * @type {!string}
 */
HTMLContentElement.prototype.select;

/**
 * @return {!NodeList}
 */
HTMLContentElement.prototype.getDistributedNodes = function() {};


/**
 * @see http://www.w3.org/TR/shadow-dom/#the-shadow-element
 * @constructor
 * @extends {HTMLElement}
 */
function HTMLShadowElement() {}

/**
 * @return {!NodeList}
 */
HTMLShadowElement.prototype.getDistributedNodes = function() {};


/**
 * @see http://www.w3.org/TR/html5/webappapis.html#the-errorevent-interface
 *
 * @constructor
 * @extends {Event}
 *
 * @param {string} type
 * @param {ErrorEventInit=} opt_eventInitDict
 */
function ErrorEvent(type, opt_eventInitDict) {}

/** @const {string} */
ErrorEvent.prototype.message;

/** @const {string} */
ErrorEvent.prototype.filename;

/** @const {number} */
ErrorEvent.prototype.lineno;

/** @const {number} */
ErrorEvent.prototype.colno;

/** @const {*} */
ErrorEvent.prototype.error;


/**
 * @see http://www.w3.org/TR/html5/webappapis.html#the-errorevent-interface
 *
 * @typedef {{
 *   bubbles: (boolean|undefined),
 *   cancelable: (boolean|undefined),
 *   message: string,
 *   filename: string,
 *   lineno: number,
 *   colno: number,
 *   error: *
 * }}
 */
 var ErrorEventInit;


/**
 * @see http://dom.spec.whatwg.org/#dom-domimplementation-createhtmldocument
 * @param {string=} opt_title A title to give the new HTML document
 * @return {!HTMLDocument}
 */
DOMImplementation.prototype.createHTMLDocument = function(opt_title) {};



/**
 * @constructor
 * @see https://html.spec.whatwg.org/multipage/embedded-content.html#the-picture-element
 * @extends {HTMLElement}
 */
function HTMLPictureElement() {}

/**
 * @constructor
 * @see https://html.spec.whatwg.org/multipage/embedded-content.html#the-picture-element
 * @extends {HTMLElement}
 */
function HTMLSourceElement() {}

/** @type {string} */
HTMLSourceElement.prototype.media;

/** @type {string} */
HTMLSourceElement.prototype.sizes;

/** @type {string} */
HTMLSourceElement.prototype.src;

/** @type {string} */
HTMLSourceElement.prototype.srcset;

/** @type {string} */
HTMLSourceElement.prototype.type;

/** @type {string} */
HTMLImageElement.prototype.sizes;

/** @type {string} */
HTMLImageElement.prototype.srcset;


/**
 * 4.11 Interactive elements
 * @see http://www.w3.org/html/wg/drafts/html/master/interactive-elements.html
 */

/**
 * @see http://www.w3.org/html/wg/drafts/html/master/interactive-elements.html#the-details-element
 * @constructor
 * @extends {HTMLElement}
 */
function HTMLDetailsElement() {}

/**
 * @see http://www.w3.org/html/wg/drafts/html/master/interactive-elements.html#dom-details-open
 * @type {boolean}
 */
HTMLDetailsElement.prototype.open;


// As of 2/20/2015, <summary> has no special web IDL interface nor global
// constructor (i.e. HTMLSummaryElement).


/**
 * @see http://www.w3.org/html/wg/drafts/html/master/interactive-elements.html#dom-menu-type
 * @type {string}
 */
HTMLMenuElement.prototype.type;

/**
 * @see http://www.w3.org/html/wg/drafts/html/master/interactive-elements.html#dom-menu-label
 * @type {string}
 */
HTMLMenuElement.prototype.label;


/**
 * @see http://www.w3.org/html/wg/drafts/html/master/interactive-elements.html#the-menuitem-element
 * @constructor
 * @extends {HTMLElement}
 */
function HTMLMenuItemElement() {}

/**
 * @see http://www.w3.org/html/wg/drafts/html/master/interactive-elements.html#dom-menuitem-type
 * @type {string}
 */
HTMLMenuItemElement.prototype.type;

/**
 * @see http://www.w3.org/html/wg/drafts/html/master/interactive-elements.html#dom-menuitem-label
 * @type {string}
 */
HTMLMenuItemElement.prototype.label;

/**
 * @see http://www.w3.org/html/wg/drafts/html/master/interactive-elements.html#dom-menuitem-icon
 * @type {string}
 */
HTMLMenuItemElement.prototype.icon;

/**
 * @see http://www.w3.org/html/wg/drafts/html/master/interactive-elements.html#dom-menuitem-disabled
 * @type {boolean}
 */
HTMLMenuItemElement.prototype.disabled;

/**
 * @see http://www.w3.org/html/wg/drafts/html/master/interactive-elements.html#dom-menuitem-checked
 * @type {boolean}
 */
HTMLMenuItemElement.prototype.checked;

/**
 * @see http://www.w3.org/html/wg/drafts/html/master/interactive-elements.html#dom-menuitem-radiogroup
 * @type {string}
 */
HTMLMenuItemElement.prototype.radiogroup;

/**
 * @see http://www.w3.org/html/wg/drafts/html/master/interactive-elements.html#dom-menuitem-default
 * @type {boolean}
 */
HTMLMenuItemElement.prototype.default;

// TODO(dbeam): add HTMLMenuItemElement.prototype.command if it's implemented.


/**
 * @see http://www.w3.org/html/wg/drafts/html/master/interactive-elements.html#relatedevent
 * @param {string} type
 * @param {{relatedTarget: (EventTarget|undefined)}=} opt_eventInitDict
 * @constructor
 * @extends {Event}
 */
function RelatedEvent(type, opt_eventInitDict) {}

/**
 * @see http://www.w3.org/html/wg/drafts/html/master/interactive-elements.html#dom-relatedevent-relatedtarget
 * @type {EventTarget|undefined}
 */
RelatedEvent.prototype.relatedTarget;


/**
 * @see http://www.w3.org/html/wg/drafts/html/master/interactive-elements.html#the-dialog-element
 * @constructor
 * @extends {HTMLElement}
 */
function HTMLDialogElement() {}

/**
 * @see http://www.w3.org/html/wg/drafts/html/master/interactive-elements.html#dom-dialog-open
 * @type {boolean}
 */
HTMLDialogElement.prototype.open;

/**
 * @see http://www.w3.org/html/wg/drafts/html/master/interactive-elements.html#dom-dialog-returnvalue
 * @type {string}
 */
HTMLDialogElement.prototype.returnValue;

/**
 * @see http://www.w3.org/html/wg/drafts/html/master/interactive-elements.html#dom-dialog-show
 * @param {(MouseEvent|Element)=} opt_anchor
 */
HTMLDialogElement.prototype.show = function(opt_anchor) {};

/**
 * @see http://www.w3.org/html/wg/drafts/html/master/interactive-elements.html#dom-dialog-showmodal
 * @param {(MouseEvent|Element)=} opt_anchor
 */
HTMLDialogElement.prototype.showModal = function(opt_anchor) {};

/**
 * @see http://www.w3.org/html/wg/drafts/html/master/interactive-elements.html#dom-dialog-close
 * @param {string=} opt_returnValue
 */
HTMLDialogElement.prototype.close = function(opt_returnValue) {};
