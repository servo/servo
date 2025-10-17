/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

/*
 * The origin of this IDL file is
 * https://www.w3.org/TR/geolocation/#coordinates_interface
 */

// https://www.w3.org/TR/geolocation/#coordinates_interface
[Pref="dom_geolocation_enabled", Exposed=Window, SecureContext]
interface GeolocationCoordinates {
  readonly attribute double accuracy;
  readonly attribute double latitude;
  readonly attribute double longitude;
  readonly attribute double? altitude;
  readonly attribute double? altitudeAccuracy;
  readonly attribute double? heading;
  readonly attribute double? speed;
};
