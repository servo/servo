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
 *  W3C's event specification by IE in JScript. This file depends on
 *  w3c_event.js.
 *
 * @see http://msdn.microsoft.com/en-us/library/ms535863.aspx
 * @externs
 */

/** @type {string} */
Event.prototype.Abstract;

/** @type {boolean} */
Event.prototype.altLeft;

/** @type {string} */
Event.prototype.Banner;

/**
 * A ClipboardData on IE, but a DataTransfer on WebKit.
 * @see http://msdn.microsoft.com/en-us/library/ms535220.aspx
 * @type {(ClipboardData|undefined)}
 */
Event.prototype.clipboardData;

/** @type {boolean} */
Event.prototype.contentOverflow;

/** @type {boolean} */
Event.prototype.ctrlLeft;

/** @type {string} */
Event.prototype.dataFld;

Event.prototype.domain;

/** @type {Element} */
Event.prototype.fromElement;

/** @type {string} */
Event.prototype.MoreInfo;

/** @type {string} */
Event.prototype.nextPage;

/** @type {number} */
Event.prototype.offsetX;

/** @type {number} */
Event.prototype.offsetY;

/** @type {string} */
Event.prototype.propertyName;

/** @type {string} */
Event.prototype.qualifier;

/** @type {number} */
Event.prototype.reason;

/** @type {Object.<*,*>} */
Event.prototype.recordset;

/** @type {boolean} */
Event.prototype.repeat;

/** @type {(boolean|string|undefined)} */
Event.prototype.returnValue;

/** @type {string} */
Event.prototype.saveType;

Event.prototype.scheme;

/** @type {boolean} */
Event.prototype.shiftLeft;

/** @type {Window} */
Event.prototype.source;

/** @type {Element} */
Event.prototype.srcElement;

Event.prototype.srcFilter;

/** @type {string} */
Event.prototype.srcUrn;

/** @type {Element} */
Event.prototype.toElement;

Event.prototype.userName;

/** @type {number} */
Event.prototype.wheelDelta;

/** @type {number} */
Event.prototype.x;

/** @type {number} */
Event.prototype.y;

/**
 * @constructor
 * @see http://msdn.microsoft.com/en-us/library/windows/apps/hh441257.aspx
 */
function MSPointerPoint() {}

/** @type {number} */
MSPointerPoint.prototype.pointerId;

/** @type {number} */
MSPointerPoint.prototype.pointerType;

/**
 * @constructor
 * @extends {Event}
 * @see http://msdn.microsoft.com/en-us/library/windows/apps/hh441233.aspx
 */
function MSPointerEvent() {}

/** @type {number} */
MSPointerEvent.MSPOINTER_TYPE_MOUSE;

/** @type {number} */
MSPointerEvent.MSPOINTER_TYPE_PEN;

/** @type {number} */
MSPointerEvent.MSPOINTER_TYPE_TOUCH;

/** @type {number} */
MSPointerEvent.prototype.height;

/** @type {number} */
MSPointerEvent.prototype.hwTimestamp;

/** @type {boolean} */
MSPointerEvent.prototype.isPrimary;

/** @type {number} */
MSPointerEvent.prototype.pointerId;

/** @type {number} */
MSPointerEvent.prototype.pointerType;

/** @type {number} */
MSPointerEvent.prototype.pressure;

/** @type {number} */
MSPointerEvent.prototype.rotation;

/** @type {number} */
MSPointerEvent.prototype.tiltX;

/** @type {number} */
MSPointerEvent.prototype.tiltY;

/** @type {number} */
MSPointerEvent.prototype.timeStamp;

/** @type {number} */
MSPointerEvent.prototype.width;

/**
 * @param {number} pointerId
 * @return {undefined}
 */
MSPointerEvent.prototype.msReleasePointerCapture;

/**
 * @param {number} pointerId
 * @return {undefined}
 */
MSPointerEvent.prototype.msSetPointerCapture;

/**
 * @param {string} typeArg
 * @param {boolean} canBubbleArg
 * @param {boolean} cancelableArg
 * @param {Window} viewArg
 * @param {number} detailArg
 * @param {number} screenXArg
 * @param {number} screenYArg
 * @param {number} clientXArg
 * @param {number} clientYArg
 * @param {boolean} ctrlKeyArg
 * @param {boolean} altKeyArg
 * @param {boolean} shiftKeyArg
 * @param {boolean} metaKeyArg
 * @param {number} buttonArg
 * @param {Element} relatedTargetArg
 * @param {number} offsetXArg
 * @param {number} offsetYArg
 * @param {number} widthArg
 * @param {number} heightArg
 * @param {number} pressure
 * @param {number} rotation
 * @param {number} tiltX
 * @param {number} tiltY
 * @param {number} pointerIdArg
 * @param {number} pointerType
 * @param {number} hwTimestampArg
 * @param {boolean} isPrimary
 * @return {undefined}
 * @see http://msdn.microsoft.com/en-us/library/windows/apps/hh441246.aspx
 */
MSPointerEvent.prototype.initPointerEvent;

/**
 * @constructor
 * @see http://msdn.microsoft.com/en-us/library/ie/hh968249(v=vs.85).aspx
 */
function MSGesture() {}

/**
 * @type {Element}
 */
MSGesture.prototype.target;

/**
 * @param {number} pointerId
 */
MSGesture.prototype.addPointer = function(pointerId) {};

MSGesture.prototype.stop = function() {};

/**
 * @constructor
 * @extends {Event}
 * @see http://msdn.microsoft.com/en-us/library/ie/hh772076(v=vs.85).aspx
 */
function MSGestureEvent() {}

/** @type {number} */
MSGestureEvent.prototype.expansion;

/** @type {!MSGesture} */
MSGestureEvent.prototype.gestureObject;

/** @type {number} */
MSGestureEvent.prototype.hwTimestamp;

/** @type {number} */
MSGestureEvent.prototype.rotation;

/** @type {number} */
MSGestureEvent.prototype.scale;

/** @type {number} */
MSGestureEvent.prototype.translationX;

/** @type {number} */
MSGestureEvent.prototype.translationY;

/** @type {number} */
MSGestureEvent.prototype.velocityAngular;

/** @type {number} */
MSGestureEvent.prototype.velocityExpansion;

/** @type {number} */
MSGestureEvent.prototype.velocityX;

/** @type {number} */
MSGestureEvent.prototype.velocityY;

/**
 * @param {string} typeArg
 * @param {boolean} canBubbleArg
 * @param {boolean} cancelableArg
 * @param {Window} viewArg
 * @param {number} detailArg
 * @param {number} screenXArg
 * @param {number} screenYArg
 * @param {number} clientXArg
 * @param {number} clientYArg
 * @param {number} offsetXArg
 * @param {number} offsetYArg
 * @param {number} translationXArg
 * @param {number} translationYArg
 * @param {number} scaleArg
 * @param {number} expansionArg
 * @param {number} rotationArg
 * @param {number} velocityXArg
 * @param {number} velocityYArg
 * @param {number} velocityExpansionArg
 * @param {number} velocityAngularArg
 * @param {number} hwTimestampArg
 * @param {EventTarget} relatedTargetArg
 * @return {undefined}
 * @see http://msdn.microsoft.com/en-us/library/windows/apps/hh441187.aspx
 */
MSGestureEvent.prototype.initGestureEvent;
