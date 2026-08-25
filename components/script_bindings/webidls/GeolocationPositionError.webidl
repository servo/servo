/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

/*
 * The origin of this IDL file is
 * https://www.w3.org/TR/geolocation/#position_error_interface
 */

// https://www.w3.org/TR/geolocation/#position_error_interface
[Pref="dom_geolocation_enabled", Exposed=Window]
interface GeolocationPositionError {
  const unsigned short PERMISSION_DENIED = 1;
  const unsigned short POSITION_UNAVAILABLE = 2;
  const unsigned short TIMEOUT = 3;
  readonly attribute unsigned short code;
  readonly attribute DOMString message;
};
