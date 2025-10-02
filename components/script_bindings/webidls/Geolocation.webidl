/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

/*
 * The origin of this IDL file is
 * https://www.w3.org/TR/geolocation/#geolocation_interface
 */

partial interface Navigator {
   [SameObject, Pref="dom_geolocation_enabled"] readonly attribute Geolocation geolocation;
};

// FIXME: The errorCallback parameters are currently commented out because they are not yet supported by the codegen,
// further debugging is needed.
// https://www.w3.org/TR/geolocation/#geolocation_interface
[Pref="dom_geolocation_enabled", Exposed=Window]
interface Geolocation {
  undefined getCurrentPosition (
    PositionCallback successCallback,
//     optional PositionErrorCallback? errorCallback = null,
    optional PositionOptions options = {}
  );

  long watchPosition (
    PositionCallback successCallback,
//     optional PositionErrorCallback? errorCallback = null,
    optional PositionOptions options = {}
  );

  undefined clearWatch (long watchId);
};

callback PositionCallback = undefined (
  GeolocationPosition position
);

callback PositionErrorCallback = undefined (
  GeolocationPositionError positionError
);

// https://www.w3.org/TR/geolocation/#position_options_interface
dictionary PositionOptions {
  boolean enableHighAccuracy = false;
  [Clamp] unsigned long timeout = 0xFFFFFFFF;
  [Clamp] unsigned long maximumAge = 0;
};
