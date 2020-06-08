﻿/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

#include "pch.h"
#include "logs.h"
#include "BrowserPage.h"
#include "BrowserPage.g.cpp"
#include "DefaultUrl.h"

using namespace std::placeholders;
using namespace winrt::Windows::Foundation;
using namespace winrt::Windows::UI::Xaml;
using namespace winrt::Windows::UI::Core;
using namespace winrt::Windows::UI::ViewManagement;
using namespace winrt::Windows::ApplicationModel::Core;
using namespace winrt::Windows::UI::Notifications;
using namespace winrt::Windows::Data::Xml::Dom;

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
  urlTextbox().GotFocus(std::bind(&BrowserPage::OnURLFocused, this, _1));
  servoControl().OnMediaSessionMetadata(
      [=](hstring title, hstring artist, hstring album) {});
  servoControl().OnMediaSessionPlaybackStateChange(
      [=](const auto &, int state) {
        if (state == static_cast<int>(servo::Servo::MediaSessionPlaybackState::None)) {
          mediaControls().Visibility(Visibility::Collapsed);
          return;
        }
        mediaControls().Visibility(Visibility::Visible);
        playButton().Visibility(
            state == static_cast<int>(servo::Servo::MediaSessionPlaybackState::Paused)
                ? Visibility::Visible
                : Visibility::Collapsed);
        pauseButton().Visibility(
            state == static_cast<int>(servo::Servo::MediaSessionPlaybackState::Paused)
                ? Visibility::Collapsed
                : Visibility::Visible);
      });
  servoControl().OnDevtoolsStatusChanged(
      [=](DevtoolsStatus status, unsigned int port) {
        mDevtoolsStatus = status;
        mDevtoolsPort = port;
      });
  Window::Current().VisibilityChanged(
      [=](const auto &, const VisibilityChangedEventArgs &args) {
        servoControl().ChangeVisibility(args.Visible());
      });
}

void BrowserPage::OnURLFocused(Windows::Foundation::IInspectable const &) {
  urlTextbox().SelectAll();
}

void BrowserPage::OnURLKeyboardAccelerator(
    Windows::Foundation::IInspectable const &,
    Windows::UI::Xaml::Input::KeyboardAcceleratorInvokedEventArgs const &) {
  urlTextbox().Focus(FocusState::Programmatic);
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

void BrowserPage::Shutdown() {
  ToastNotificationManager::History().Clear();
  servoControl().Shutdown();
}

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

void BrowserPage::OnHomeButtonClicked(IInspectable const &,
                                      RoutedEventArgs const &) {
  servoControl().LoadURIOrSearch(DEFAULT_URL);
}

void BrowserPage::OnDevtoolsButtonClicked(IInspectable const &,
                                          RoutedEventArgs const &) {
  auto toastTemplate = ToastTemplateType::ToastText01;
  auto toastXml = ToastNotificationManager::GetTemplateContent(toastTemplate);
  auto toastTextElements = toastXml.GetElementsByTagName(L"text");
  std::wstring message;
  if (mDevtoolsStatus == DevtoolsStatus::Stopped) {
    message = L"Devtools server hasn't started";
  } else if (mDevtoolsStatus == DevtoolsStatus::Running) {
    message = L"DevTools server has started on port " +
              std::to_wstring(mDevtoolsPort);
  } else if (mDevtoolsStatus == DevtoolsStatus::Failed) {
    message = L"Error: could not start DevTools";
  }
  toastTextElements.Item(0).InnerText(message);
  auto toast = ToastNotification(toastXml);
  ToastNotificationManager::CreateToastNotifier().Show(toast);
}

void BrowserPage::OnURLEdited(IInspectable const &,
                              Input::KeyRoutedEventArgs const &e) {
  if (e.Key() == Windows::System::VirtualKey::Enter) {
    servoControl().Focus(FocusState::Programmatic);
    auto input = urlTextbox().Text();
    auto uri = servoControl().LoadURIOrSearch(input);
    urlTextbox().Text(uri);
  }
}

void BrowserPage::OnMediaControlsPlayClicked(
    Windows::Foundation::IInspectable const &,
    Windows::UI::Xaml::RoutedEventArgs const &) {
  servoControl().SendMediaSessionAction(
      static_cast<int32_t>(servo::Servo::MediaSessionActionType::Play));
}
void BrowserPage::OnMediaControlsPauseClicked(
    Windows::Foundation::IInspectable const &,
    Windows::UI::Xaml::RoutedEventArgs const &) {
  servoControl().SendMediaSessionAction(
      static_cast<int32_t>(servo::Servo::MediaSessionActionType::Pause));
}

} // namespace winrt::ServoApp::implementation
