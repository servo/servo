/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

// Minimal header-only C++ helpers for the Servo C API.
//
// The C API (see `servo_capi.h`) hands out owned handles that the caller must
// release again with the matching `servo_*_free` function. The aliases below
// wrap those handles in `std::unique_ptr`, so they are released automatically
// when they go out of scope and cannot be copied by accident (which would lead
// to a double free). Use `.get()` to pass the underlying pointer to the C API
// and `.release()` when a call transfers ownership to Servo.
//
// Usage:
//
//     #include "servo/servo_capi.h"
//     #include "servo_capi.hpp"
//
//     servo::ServoOptions options(servo_options_create());
//     if (!options) { /* handle allocation failure */ }
//
// This header requires C++17 and must be compiled with the generated
// `servo_capi.h` available on the include path (e.g. `-I <prefix>/include`).

#ifndef SERVO_CAPI_HPP
#define SERVO_CAPI_HPP

#include <memory>
#include <utility>

#include "servo/servo_capi.h"

namespace servo {

// Stateless deleter that forwards to the matching `servo_*_free` function.
// Some `servo_*_free` functions return a `ServoError`; the value is
// intentionally ignored here, matching how the C API is meant to be used.
template <typename T, auto FreeFunction>
struct Deleter {
    void operator()(T* pointer) const noexcept {
        if (pointer != nullptr) {
            FreeFunction(pointer);
        }
    }
};

// An owning, move-only handle to an object created by the Servo C API.
template <typename T, auto FreeFunction>
using Handle = std::unique_ptr<T, Deleter<T, FreeFunction>>;

using Servo = Handle<::Servo, servo_free>;
using ServoBuilder = Handle<::ServoBuilder, servo_builder_free>;
using ServoOptions = Handle<::ServoOptions, servo_options_free>;
using ServoPreferences = Handle<::ServoPreferences, servo_preferences_free>;
using ServoRequestContext = Handle<::ServoRequestContext, servo_request_context_free>;
using RenderingContext = Handle<::RenderingContext, servo_rendering_context_free>;
using ServoWebViewBuilder = Handle<::ServoWebViewBuilder, servo_webview_builder_free>;
using WebView = Handle<::WebView, servo_webview_free>;
using ServoUserContentManager =
    Handle<::ServoUserContentManager, servo_user_content_manager_free>;
using ServoUserScript = Handle<::ServoUserScript, servo_user_script_free>;
using ServoUserStyleSheet = Handle<::ServoUserStyleSheet, servo_user_style_sheet_free>;
using ServoValue = Handle<::ServoValue, servo_value_free>;
using ServoDictionaryValue = Handle<::ServoDictionaryValue, servo_dictionary_free>;
using ServoListValue = Handle<::ServoListValue, servo_list_free>;
using ServoNetworkManager = Handle<::ServoNetworkManager, servo_network_manager_free>;
using ServoSiteDataManager = Handle<::ServoSiteDataManager, servo_site_data_manager_free>;
using ServoWebResourceRequest =
    Handle<::ServoWebResourceRequest, servo_web_resource_request_free>;
using ServoWebResourceResponse =
    Handle<::ServoWebResourceResponse, servo_web_resource_response_free>;
using ServoCreateNewWebViewRequest =
    Handle<::ServoCreateNewWebViewRequest, servo_create_new_webview_request_free>;

}  // namespace servo

#endif  // SERVO_CAPI_HPP
