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
 * @fileoverview Definitions for all the extensions over
 *  W3C's event specification by Gecko. This file depends on
 *  w3c_event.js.
 *
 * @externs
 */

// TODO: Almost all of it has not been annotated with types.

/** @type {number} */ Event.prototype.HORIZONTAL_AXIS;
/** @type {number} */ Event.prototype.VERTICAL_AXIS;
/** @type {boolean} */ Event.prototype.altKey;
/** @type {number} */ Event.prototype.axis;
/** @type {number} */ Event.prototype.button;
/** @type {boolean} */ Event.prototype.cancelBubble;
/** @type {number} */ Event.prototype.charCode;
/** @type {number} */ Event.prototype.clientX;
/** @type {number} */ Event.prototype.clientY;
/** @type {boolean} */ Event.prototype.ctrlKey;
/** @type {EventTarget} */ Event.prototype.explicitOriginalTarget;
/** @type {boolean} */ Event.prototype.isChar;
/** @type {number} */ Event.prototype.keyCode;
/** @type {number} */ Event.prototype.layerX;
/** @type {number} */ Event.prototype.layerY;
/** @type {boolean} */ Event.prototype.metaKey;
/** @type {EventTarget} */ Event.prototype.originalTarget;
/** @type {number} */ Event.prototype.pageX;
/** @type {number} */ Event.prototype.pageY;
/** @type {EventTarget} */ Event.prototype.relatedTarget;
/** @type {number} */ Event.prototype.screenX;
/** @type {number} */ Event.prototype.screenY;
/** @type {boolean} */ Event.prototype.shiftKey;
/** @type {Window} */ Event.prototype.view;
/** @type {number} */ Event.prototype.which;

/** @constructor */ function nsIDOMPageTransitionEvent() {}
/** @type {boolean} */ nsIDOMPageTransitionEvent.prototype.persisted;

//Methods
Event.prototype.initKeyEvent;
Event.prototype.initMouseEvent;
Event.prototype.initUIEvent;
Event.prototype.initMessageEvent;
Event.prototype.preventBubble;
Event.prototype.preventCapture;
