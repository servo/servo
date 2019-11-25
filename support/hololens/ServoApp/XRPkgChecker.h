/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

#pragma once

#include "pch.h"

namespace winrt {
class XRPkgChecker {

public:
  void OnInstalled(std::function<void()> callback,
                   winrt::Windows::Foundation::TimeSpan interval);
  bool IsInstalled();
  void StopTracking();
  void OpenStore();

private:
  std::unique_ptr<std::function<void()>> installed_callback;
  void CheckXRPkgTick(Windows::Foundation::IInspectable const &,
                      Windows::Foundation::IInspectable const &);
  Windows::UI::Xaml::DispatcherTimer timer;
  inline static const hstring OPENXR_PACKAGE_NAME =
      L"Microsoft.MixedRealityRuntimeDeveloperPreview_8wekyb3d8bbwe";
  inline static const hstring OPENXR_PACKAGE_SHORT_NAME =
      L"Microsoft.MixedRealityRuntimeDeveloperPreview";
};

} // namespace winrt
