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
 * @fileoverview Definitions for W3C's Geolocation specification
 *     http://www.w3.org/TR/geolocation-API/
 * @externs
 */

/**
 * @constructor
 * @see http://www.w3.org/TR/geolocation-API/#geolocation
 */
function Geolocation() {}

/**
 * @param {function(GeolocationPosition)} successCallback
 * @param {(function(GeolocationPositionError)|null)=} opt_errorCallback
 * @param {GeolocationPositionOptions=} opt_options
 */
Geolocation.prototype.getCurrentPosition = function(successCallback,
                                                       opt_errorCallback,
                                                       opt_options) {};

/**
 * @param {function(GeolocationPosition)} successCallback
 * @param {(function(GeolocationPositionError)|null)=} opt_errorCallback
 * @param {GeolocationPositionOptions=} opt_options
 * @return {number}
 */
Geolocation.prototype.watchPosition = function(successCallback,
                                                  opt_errorCallback,
                                                  opt_options) {};

/** @param {number} watchId */
Geolocation.prototype.clearWatch = function(watchId) {};


/**
 * @constructor
 * @see http://www.w3.org/TR/geolocation-API/#coordinates
 */
function GeolocationCoordinates() {}
/** @type {number} */ GeolocationCoordinates.prototype.latitude;
/** @type {number} */ GeolocationCoordinates.prototype.longitude;
/** @type {number} */ GeolocationCoordinates.prototype.accuracy;
/** @type {number} */ GeolocationCoordinates.prototype.altitude;
/** @type {number} */ GeolocationCoordinates.prototype.altitudeAccuracy;
/** @type {number} */ GeolocationCoordinates.prototype.heading;
/** @type {number} */ GeolocationCoordinates.prototype.speed;


/**
 * @constructor
 * @see http://www.w3.org/TR/geolocation-API/#position
 */
function GeolocationPosition() {}
/** @type {GeolocationCoordinates} */
GeolocationPosition.prototype.coords;
/** @type {Date} */ GeolocationPosition.prototype.timestamp;


/**
 * @constructor
 * @see http://www.w3.org/TR/geolocation-API/#position-options
 */
function GeolocationPositionOptions() {}
/** @type {boolean} */
GeolocationPositionOptions.prototype.enableHighAccuracy;
/** @type {number} */ GeolocationPositionOptions.prototype.maximumAge;
/** @type {number} */ GeolocationPositionOptions.prototype.timeout;


/**
 * @constructor
 * @see http://www.w3.org/TR/geolocation-API/#position-error
 */
function GeolocationPositionError() {}
/** @type {number} */ GeolocationPositionError.prototype.code;
/** @type {string} */ GeolocationPositionError.prototype.message;
/** @type {number} */ GeolocationPositionError.prototype.UNKNOWN_ERROR;
/** @type {number} */ GeolocationPositionError.prototype.PERMISSION_DENIED;
/** @type {number} */
GeolocationPositionError.prototype.POSITION_UNAVAILABLE;
/** @type {number} */ GeolocationPositionError.prototype.TIMEOUT;

/** @type {Geolocation} */
Navigator.prototype.geolocation;
