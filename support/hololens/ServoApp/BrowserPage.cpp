/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

#include "pch.h"
#include "logs.h"
#include "BrowserPage.h"
#include "BrowserPage.g.cpp"
#include "ImmersiveView.h"
#include "OpenGLES.h"

using namespace winrt::Windows::UI::Xaml;
using namespace winrt::Windows::UI::Core;
using namespace winrt::Windows::UI::ViewManagement;
using namespace winrt::Windows::Graphics::Holographic;
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
    reloadButton().IsEnabled(false);
    stopButton().IsEnabled(true);
  });
  servoControl().OnLoadEnded([=] {
    reloadButton().IsEnabled(true);
    stopButton().IsEnabled(false);
  });
}

void BrowserPage::Shutdown() {
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

void BrowserPage::OnURLEdited(IInspectable const &sender,
                              Input::KeyRoutedEventArgs const &e) {
  if (e.Key() == Windows::System::VirtualKey::Enter) {
    servoControl().Focus(FocusState::Programmatic);
    auto input = urlTextbox().Text();
    auto uri = servoControl().LoadURIOrSearch(input);
    urlTextbox().Text(uri.ToString());
  }
}

void BrowserPage::OnImmersiveButtonClicked(IInspectable const &,
                                           RoutedEventArgs const &) {
  if (HolographicSpace::IsAvailable()) {
    log("Holographic space available");
    auto v = CoreApplication::CreateNewView(mImmersiveViewSource);
    auto parentId = ApplicationView::GetForCurrentView().Id();
    v.Dispatcher().RunAsync(CoreDispatcherPriority::Normal, [=] {
      auto winId = ApplicationView::GetForCurrentView().Id();
      ApplicationViewSwitcher::SwitchAsync(winId, parentId);
      log("Immersive view started");
    });
  } else {
    log("Holographic space not available");
  }
}

} // namespace winrt::ServoApp::implementation
