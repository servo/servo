/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

#include "pch.h"
#include "XRPkgChecker.h"
#include "logs.h"
#include "winrt/Windows.Management.Deployment.h"

using namespace winrt::Windows::Management::Deployment;

namespace winrt {

void XRPkgChecker::OnInstalled(std::function<void()> callback,
                               winrt::Windows::Foundation::TimeSpan interval) {
  timer.Stop();
  timer.Interval(interval);
  installed_callback = std::make_unique<std::function<void()>>(callback);
  timer.Tick({this, &XRPkgChecker::CheckXRPkgTick});
  timer.Start();
}

void XRPkgChecker::StopTracking() {
  installed_callback.reset();
  timer.Stop();
}

void XRPkgChecker::CheckXRPkgTick(Windows::Foundation::IInspectable const &,
                                  Windows::Foundation::IInspectable const &) {
  if (IsInstalled()) {
    (*installed_callback)();
    StopTracking();
  }
}

void XRPkgChecker::OpenStore() {
  std::wstring url = L"ms-windows-store://pdp/?PFN=";
  Windows::Foundation::Uri uri{url + OPENXR_PACKAGE_NAME};
  Windows::System::Launcher::LaunchUriAsync(uri);
}

bool XRPkgChecker::IsInstalled() {
  auto current_user = L"";
  for (auto package : PackageManager().FindPackagesForUser(current_user)) {
    if (package.Id().Name() == OPENXR_PACKAGE_SHORT_NAME) {
      return true;
    }
  }
  return false;
}

} // namespace winrt
