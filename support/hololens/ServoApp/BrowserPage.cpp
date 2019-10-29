/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

#include "pch.h"
#include "logs.h"
#include "BrowserPage.h"
#include "BrowserPage.g.cpp"

using namespace winrt::Windows::Foundation;
using namespace winrt::Windows::UI::Xaml;
using namespace winrt::Windows::UI::Core;
using namespace winrt::Windows::UI::ViewManagement;
using namespace winrt::Windows::ApplicationModel::Core;

namespace winrt::ServoApp::implementation {
BrowserPage::BrowserPage() {
  InitializeComponent();
  BindServoEvents();
}

void BrowserPage::BindServoEvents() {
  servoControl().OnURLChanged(
      [=](const auto &, hstring url) { urlTextbox().Text(url); });
  servoControl().OnTitleChanged([=](const auto &, hstring title) {});
  servoControl().OnHistoryChanged([=](bool back, bool forward) {
    backButton().IsEnabled(back);
    forwardButton().IsEnabled(forward);
  });
  servoControl().OnLoadStarted([=] {
    urlbarLoadingIndicator().IsActive(true);
    transientLoadingIndicator().IsIndeterminate(true);

    reloadButton().IsEnabled(false);
    reloadButton().Visibility(Visibility::Collapsed);
    stopButton().IsEnabled(true);
    stopButton().Visibility(Visibility::Visible);
  });
  servoControl().OnLoadEnded([=] {
    urlbarLoadingIndicator().IsActive(false);
    transientLoadingIndicator().IsIndeterminate(false);
    reloadButton().IsEnabled(true);
    reloadButton().Visibility(Visibility::Visible);
    stopButton().IsEnabled(false);
    stopButton().Visibility(Visibility::Collapsed);
  });
  servoControl().OnCaptureGesturesStarted([=] {
    servoControl().Focus(FocusState::Programmatic);
    navigationBar().IsHitTestVisible(false);
  });
  servoControl().OnCaptureGesturesEnded(
      [=] { navigationBar().IsHitTestVisible(true); });
}

void BrowserPage::LoadServoURI(Uri uri) {
  auto scheme = uri.SchemeName();

  if (scheme != SERVO_SCHEME) {
    log("Unexpected URL: ", uri.RawUri().c_str());
    return;
  }
  std::wstring raw{uri.RawUri()};
  auto raw2 = raw.substr(SERVO_SCHEME_SLASH_SLASH.size());
  servoControl().LoadURIOrSearch(raw2);
}

void BrowserPage::SetTransientMode(bool transient) {
  servoControl().SetTransientMode(transient);
  navigationBar().Visibility(transient ? Visibility::Collapsed
                                       : Visibility::Visible);
  transientLoadingIndicator().Visibility(transient ? Visibility::Visible
                                            : Visibility::Collapsed);
}

void BrowserPage::SetArgs(hstring args) { servoControl().SetArgs(args); }

void BrowserPage::Shutdown() { servoControl().Shutdown(); }

/**** USER INTERACTIONS WITH UI ****/

void BrowserPage::OnBackButtonClicked(IInspectable const &,
                                      RoutedEventArgs const &) {
  servoControl().GoBack();
}

void BrowserPage::OnForwardButtonClicked(IInspectable const &,
                                         RoutedEventArgs const &) {
  servoControl().GoForward();
}

void BrowserPage::OnReloadButtonClicked(IInspectable const &,
                                        RoutedEventArgs const &) {
  servoControl().Reload();
}

void BrowserPage::OnStopButtonClicked(IInspectable const &,
                                      RoutedEventArgs const &) {
  servoControl().Stop();
}

void BrowserPage::OnURLEdited(IInspectable const &,
                              Input::KeyRoutedEventArgs const &e) {
  if (e.Key() == Windows::System::VirtualKey::Enter) {
    servoControl().Focus(FocusState::Programmatic);
    auto input = urlTextbox().Text();
    auto uri = servoControl().LoadURIOrSearch(input);
    urlTextbox().Text(uri.ToString());
  }
}

} // namespace winrt::ServoApp::implementation
