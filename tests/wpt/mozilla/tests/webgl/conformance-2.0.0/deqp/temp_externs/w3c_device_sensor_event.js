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
 * @fileoverview Definitions for W3C's device orientation and device motion
 *  events specification.
 *  This file depends on w3c_event.js.
 *  The whole file has been partially type annotated.
 *  Created from http://dev.w3.org/geo/api/spec-source-orientation.
 *
 * @externs
 */

/**
 * @constructor
 * @extends {Event}
 */
function DeviceOrientationEvent() {}

/** @type {?number} */
DeviceOrientationEvent.prototype.alpha;

/** @type {?number} */
DeviceOrientationEvent.prototype.beta;

/** @type {?number} */
DeviceOrientationEvent.prototype.gamma;

/** @type {boolean} */
DeviceOrientationEvent.prototype.absolute;

/**
 * @type {?number}
 * @see https://developer.apple.com/library/safari/documentation/SafariDOMAdditions/Reference/DeviceOrientationEventClassRef/DeviceOrientationEvent/DeviceOrientationEvent.html#//apple_ref/javascript/instp/DeviceOrientationEvent/webkitCompassAccuracy
 */
DeviceOrientationEvent.prototype.webkitCompassAccuracy;

/**
 * @type {?number}
 * @see https://developer.apple.com/library/safari/documentation/SafariDOMAdditions/Reference/DeviceOrientationEventClassRef/DeviceOrientationEvent/DeviceOrientationEvent.html#//apple_ref/javascript/instp/DeviceOrientationEvent/webkitCompassHeading
 */
DeviceOrientationEvent.prototype.webkitCompassHeading;

/**
 * @constructor
 */
function DeviceAcceleration() {}

/** @type {?number} */
DeviceAcceleration.prototype.x;

/** @type {?number} */
DeviceAcceleration.prototype.y;

/** @type {?number} */
DeviceAcceleration.prototype.z;

/**
 * @constructor
 */
function DeviceRotationRate() {}

/** @type {?number} */
DeviceRotationRate.prototype.alpha;

/** @type {?number} */
DeviceRotationRate.prototype.beta;

/** @type {?number} */
DeviceRotationRate.prototype.gamma;

/**
 * @constructor
 * @extends {Event}
 */
function DeviceMotionEvent() {}

/** @type {?DeviceAcceleration} */
DeviceMotionEvent.prototype.acceleration;

/** @type {?DeviceAcceleration} */
DeviceMotionEvent.prototype.accelerationIncludingGravity;

/** @type {?DeviceRotationRate} */
DeviceMotionEvent.prototype.rotationRate;

/** @type {?number} */
DeviceMotionEvent.prototype.interval;
