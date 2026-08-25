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
 * @fileoverview Definitions for all the extensions over W3C's
 *  event specification by WebKit. This file depends on w3c_event.js.
 *  All the provided definitions have been type annotated
 *
 * @externs
 */

/** @type {number} */
Event.prototype.wheelDeltaX;

/** @type {number} */
Event.prototype.wheelDeltaY;

/**
 * @constructor
 * @extends {Event}
 * @see http://developer.apple.com/library/safari/documentation/AudioVideo/Reference/WebKitAnimationEventClassReference/WebKitAnimationEvent/WebKitAnimationEvent.html
 */
function WebKitAnimationEvent() {}

/**
 * @type {string}
 * @const
 */
WebKitAnimationEvent.prototype.animationName;

/**
 * @type {number}
 * @const
 */
WebKitAnimationEvent.prototype.elapsedTime;