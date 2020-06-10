/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

#include "pch.h"
#include "logs.h"
#include "BrowserPage.h"
#include "BrowserPage.g.cpp"
#include "DefaultUrl.h"

#include "winrt/Microsoft.UI.Xaml.Controls.h"
#include "winrt/Microsoft.UI.Xaml.XamlTypeInfo.h"
#include "winrt/Windows.UI.Text.h"
#include "winrt/Windows.UI.Xaml.Documents.h" // For Run.Text()

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
        if (state == servo::Servo::MediaSessionPlaybackState::None) {
          mediaControls().Visibility(Visibility::Collapsed);
          return;
        }
        mediaControls().Visibility(Visibility::Visible);
        playButton().Visibility(
            state == servo::Servo::MediaSessionPlaybackState::Paused
                ? Visibility::Visible
                : Visibility::Collapsed);
        pauseButton().Visibility(
            state == servo::Servo::MediaSessionPlaybackState::Paused
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

void BrowserPage::OnURLFocused(IInspectable const &) {
  urlTextbox().SelectAll();
}

void BrowserPage::OnURLKeyboardAccelerator(
    IInspectable const &, Input::KeyboardAcceleratorInvokedEventArgs const &) {
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

void BrowserPage::OnHomeButtonClicked(IInspectable const &,
                                      RoutedEventArgs const &) {
  servoControl().LoadURIOrSearch(DEFAULT_URL);
}

// Given a pref, update its associated UI control.
void BrowserPage::UpdatePref(ServoApp::Pref pref, Controls::Control ctrl) {
  auto value = pref.Value();
  auto type = value.as<IPropertyValue>().Type();
  if (type == PropertyType::Boolean) {
    ctrl.as<Controls::CheckBox>().IsChecked(unbox_value<bool>(value));
  } else if (type == PropertyType::Double) {
    ctrl.as<Microsoft::UI::Xaml::Controls::NumberBox>().Value(
        unbox_value<double>(value));
  } else if (type == PropertyType::Int64) {
    ctrl.as<Microsoft::UI::Xaml::Controls::NumberBox>().Value(
        (double)unbox_value<int64_t>(value));
  } else if (type == PropertyType::String) {
    ctrl.as<Controls::TextBox>().Text(unbox_value<hstring>(value));
  }
  auto stack = ctrl.Parent().as<Controls::StackPanel>();
  auto font = winrt::Windows::UI::Text::FontWeights::Normal();
  if (!pref.IsDefault()) {
    font = winrt::Windows::UI::Text::FontWeights::Bold();
  }
  stack.Children().GetAt(0).as<Controls::TextBlock>().FontWeight(font);
  stack.Children().GetAt(2).as<Controls::Button>().IsEnabled(!pref.IsDefault());
}

// Retrieve the preference list from Servo and build the preference table.
void BrowserPage::BuildPrefList() {
  // It would be better to use a template and bindings, but the
  // <ListView> takes too long to generate all the items, and
  // it's pretty difficiult to have different controls depending
  // on the pref type.
  prefList().Children().Clear();
  for (auto pref : ServoControl().Preferences()) {
    auto value = pref.Value();
    auto type = value.as<IPropertyValue>().Type();
    std::optional<Controls::Control> ctrl;
    if (type == PropertyType::Boolean) {
      auto checkbox = Controls::CheckBox();
      checkbox.IsChecked(unbox_value<bool>(value));
      checkbox.Click([=](const auto &, auto const &) {
        auto upref = ServoControl().SetBoolPref(
            pref.Key(), checkbox.IsChecked().GetBoolean());
        UpdatePref(upref, checkbox);
      });
      ctrl = checkbox;
    } else if (type == PropertyType::String) {
      auto textbox = Controls::TextBox();
      textbox.Text(unbox_value<hstring>(value));
      textbox.KeyUp([=](const auto &, Input::KeyRoutedEventArgs const &e) {
        if (e.Key() == Windows::System::VirtualKey::Enter) {
          auto upref = ServoControl().SetStringPref(pref.Key(), textbox.Text());
          UpdatePref(upref, textbox);
        }
      });
      ctrl = textbox;
    } else if (type == PropertyType::Int64) {
      // Note: These are *not* under Windows::UI:Xaml namespace.
      auto nbox = Microsoft::UI::Xaml::Controls::NumberBox();
      nbox.Value((double)unbox_value<int64_t>(value));
      nbox.SpinButtonPlacementMode(
          Microsoft::UI::Xaml::Controls::NumberBoxSpinButtonPlacementMode::
              Inline);
      nbox.ValueChanged([=](const auto &, const auto &) {
        int val = (int)nbox.Value();
        auto upref = ServoControl().SetIntPref(pref.Key(), val);
        UpdatePref(upref, nbox);
      });
      ctrl = nbox;
    } else if (type == PropertyType::Double) {
      auto nbox = Microsoft::UI::Xaml::Controls::NumberBox();
      nbox.Value(unbox_value<double>(value));
      nbox.ValueChanged([=](const auto &, const auto &) {
        auto upref =
            ServoControl().SetIntPref(pref.Key(), (int64_t)nbox.Value());
        UpdatePref(upref, (Controls::Control &)nbox);
      });
      ctrl = nbox;
    }
    if (ctrl.has_value()) {
      auto stack = Controls::StackPanel();
      stack.Tag(winrt::box_value(pref.Key()));
      stack.Padding({4, 4, 4, 4});
      stack.Orientation(Controls::Orientation::Horizontal);
      auto key = Controls::TextBlock();
      key.Text(pref.Key());
      key.Width(350);
      if (!pref.IsDefault()) {
        auto font = winrt::Windows::UI::Text::FontWeights::Bold();
        key.FontWeight(font);
      }
      stack.Children().Append(key);
      ctrl->Width(300);
      ctrl->Margin({4, 0, 40, 0});
      stack.Children().Append(*ctrl);
      auto reset = Controls::Button();
      reset.Content(winrt::box_value(L"reset"));
      reset.IsEnabled(!pref.IsDefault());
      reset.Click([=](const auto &, auto const &) {
        auto upref = ServoControl().ResetPref(pref.Key());
        UpdatePref(upref, *ctrl);
      });
      stack.Children().Append(reset);
      prefList().Children().Append(stack);
    }
  }
}

void BrowserPage::OnPrefererenceSearchboxEdited(
    IInspectable const &, Input::KeyRoutedEventArgs const &) {
  auto input = preferenceSearchbox().Text();
  for (auto element : prefList().Children()) {
    auto ctrl = (Controls::Control &)element;
    if (input.size() == 0) {
      ctrl.Visibility(Visibility::Visible);
    } else {
      auto tag = ctrl.Tag();
      std::wstring key = static_cast<std::wstring>(unbox_value<hstring>(tag));
      bool not_found = key.find(input) == std::wstring::npos;
      ctrl.Visibility(not_found ? Visibility::Collapsed : Visibility::Visible);
    }
  }
}

void BrowserPage::OnDevtoolsButtonClicked(IInspectable const &,
                                          RoutedEventArgs const &) {
  if (toolbox().Visibility() == Visibility::Visible) {
    prefList().Children().Clear();
    toolbox().Visibility(Visibility::Collapsed);
    return;
  }

  toolbox().Visibility(Visibility::Visible);

  BuildPrefList();

  // FIXME: we could use template + binding for this.
  auto ok = mDevtoolsStatus == DevtoolsStatus::Running ? Visibility::Visible
                                                       : Visibility::Collapsed;
  auto ko = mDevtoolsStatus == DevtoolsStatus::Failed ? Visibility::Visible
                                                      : Visibility::Collapsed;
  auto wip = mDevtoolsStatus == DevtoolsStatus::Stopped ? Visibility::Visible
                                                        : Visibility::Collapsed;
  DevtoolsStatusOK().Visibility(ok);
  DevtoolsStatusKO().Visibility(ko);
  DevtoolsStatusWIP().Visibility(wip);
  if (mDevtoolsStatus == DevtoolsStatus::Running) {
    DevtoolsPort().Text(std::to_wstring(mDevtoolsPort));
  }
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

void BrowserPage::OnMediaControlsPlayClicked(IInspectable const &,
                                             RoutedEventArgs const &) {
  servoControl().SendMediaSessionAction(
      static_cast<int32_t>(servo::Servo::MediaSessionActionType::Play));
}
void BrowserPage::OnMediaControlsPauseClicked(IInspectable const &,
                                              RoutedEventArgs const &) {
  servoControl().SendMediaSessionAction(
      static_cast<int32_t>(servo::Servo::MediaSessionActionType::Pause));
}

} // namespace winrt::ServoApp::implementation
