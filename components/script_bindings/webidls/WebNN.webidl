/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

// Source: Web Neural Network API (https://www.w3.org/TR/webnn/#navigatorml)

// skip-unless CARGO_FEATURE_WEBGPU

interface mixin NavigatorML {
  [SecureContext, SameObject, Pref="dom_webnn_enabled"] readonly attribute ML ml;
};
Navigator includes NavigatorML;
WorkerNavigator includes NavigatorML;

dictionary MLContextOptions {
  boolean accelerated = true;
};

[Exposed=(Window, Worker), SecureContext, Pref="dom_webnn_enabled"]
interface ML {
  Promise<MLContext> createContext(optional MLContextOptions options = {});
};

[Exposed=(Window, Worker), SecureContext, Pref="dom_webnn_enabled"]
interface MLContext {
};
