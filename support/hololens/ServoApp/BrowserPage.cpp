/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

#include "pch.h"
#include "strutils.h"
#include "BrowserPage.h"
#include "BrowserPage.g.cpp"
#include "ConsoleLog.g.cpp"
#include "Devtools/Client.h"

using namespace std::placeholders;
using namespace winrt::Windows::Foundation;
using namespace winrt::Windows::UI::Xaml;
using namespace winrt::Windows::UI::Core;
using namespace winrt::Windows::UI::ViewManagement;
using namespace winrt::Windows::ApplicationModel::Core;
using namespace winrt::Windows::ApplicationModel::Resources;
using namespace winrt::Windows::UI::Notifications;
using namespace winrt::Windows::Data::Json;
using namespace winrt::Windows::Data::Xml::Dom;
using namespace winrt::servo;

namespace winrt::ServoApp::implementation {

BrowserPage::BrowserPage() {
  InitializeComponent();
  BindServoEvents();
  mLogs = winrt::single_threaded_observable_vector<IInspectable>();
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
    devtoolsButton().IsEnabled(true);
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
  servoControl().OnMediaSessionPlaybackStateChange([=](const auto &,
                                                       int state) {
    if (state == Servo::MediaSessionPlaybackState::None) {
      mediaControls().Visibility(Visibility::Collapsed);
      return;
    }
    mediaControls().Visibility(Visibility::Visible);
    playButton().Visibility(state == Servo::MediaSessionPlaybackState::Paused
                                ? Visibility::Visible
                                : Visibility::Collapsed);
    pauseButton().Visibility(state == Servo::MediaSessionPlaybackState::Paused
                                 ? Visibility::Collapsed
                                 : Visibility::Visible);
  });
  servoControl().OnDevtoolsStatusChanged(
      [=](DevtoolsStatus status, unsigned int port, hstring token) {
        mDevtoolsStatus = status;
        mDevtoolsPort = port;
        mDevtoolsToken = token;
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
    log(L"Unexpected URL: ", uri.RawUri().c_str());
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
  servoControl().GoHome();
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
  auto resourceLoader = ResourceLoader::GetForCurrentView();
  auto resetStr =
      resourceLoader.GetString(L"devtoolsPreferenceResetButton/Content");
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
      reset.Content(winrt::box_value(resetStr));
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

void BrowserPage::ClearConsole() {
  Dispatcher().RunAsync(CoreDispatcherPriority::High, [=] { mLogs.Clear(); });
}

void BrowserPage::OnDevtoolsMessage(DevtoolsMessageLevel level, hstring source,
                                    hstring body) {
  Dispatcher().RunAsync(CoreDispatcherPriority::High, [=] {
    auto glyphColor = UI::Colors::Transparent();
    auto glyph = L"";
    if (level == servo::DevtoolsMessageLevel::Error) {
      glyphColor = UI::Colors::Red();
      glyph = L"\xEA39"; // ErrorBadge
    } else if (level == servo::DevtoolsMessageLevel::Warn) {
      glyphColor = UI::Colors::Orange();
      glyph = L"\xE7BA"; // Warning
    }
    mLogs.Append(make<ConsoleLog>(glyphColor, glyph, body, source));
  });
}

void BrowserPage::OnDevtoolsDetached() {}

void BrowserPage::OnDevtoolsButtonClicked(IInspectable const &,
                                          RoutedEventArgs const &) {
  if (toolbox().Visibility() == Visibility::Visible) {
    prefList().Children().Clear();
    toolbox().Visibility(Visibility::Collapsed);
    ClearConsole();
    if (mDevtoolsClient != nullptr) {
      mDevtoolsClient->Stop();
    }
    return;
  }

  toolbox().Visibility(Visibility::Visible);

  BuildPrefList();

  auto resourceLoader = ResourceLoader::GetForCurrentView();
  if (mDevtoolsStatus == DevtoolsStatus::Running) {
    hstring port = to_hstring(mDevtoolsPort);
    if (mDevtoolsClient == nullptr) {
      DevtoolsDelegate *dd = static_cast<DevtoolsDelegate *>(this);
      mDevtoolsClient = std::make_unique<DevtoolsClient>(L"localhost", port,
                                                         mDevtoolsToken, *dd);
    }
    mDevtoolsClient->Run();
    std::wstring message =
        resourceLoader.GetString(L"devtoolsStatus/Running").c_str();
    hstring formatted{format(message, port.c_str())};
    OnDevtoolsMessage(servo::DevtoolsMessageLevel::None, L"", formatted);
  } else if (mDevtoolsStatus == DevtoolsStatus::Failed) {
    auto body = resourceLoader.GetString(L"devtoolsStatus/Failed");
    OnDevtoolsMessage(servo::DevtoolsMessageLevel::Error, L"", body);
  } else if (mDevtoolsStatus == DevtoolsStatus::Stopped) {
    auto body = resourceLoader.GetString(L"devtoolsStatus/Stopped");
    OnDevtoolsMessage(servo::DevtoolsMessageLevel::None, L"", body);
  }
}

void BrowserPage::OnJSInputEdited(IInspectable const &,
                                  Input::KeyRoutedEventArgs const &e) {
  if (e.Key() == Windows::System::VirtualKey::Enter) {
    auto input = JSInput().Text();
    JSInput().Text(L"");
    mDevtoolsClient->Evaluate(input);
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
      static_cast<int32_t>(Servo::MediaSessionActionType::Play));
}
void BrowserPage::OnMediaControlsPauseClicked(IInspectable const &,
                                              RoutedEventArgs const &) {
  servoControl().SendMediaSessionAction(
      static_cast<int32_t>(Servo::MediaSessionActionType::Pause));
}

} // namespace winrt::ServoApp::implementation
