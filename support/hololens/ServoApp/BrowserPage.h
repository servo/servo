/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

#pragma once

#include "BrowserPage.g.h"
#include "Bookmark.g.h"
#include "ConsoleLog.g.h"
#include "ServoControl/ServoControl.h"
#include "Devtools/Client.h"
#include "Bookmarks.h"

namespace winrt::ServoApp::implementation {

using namespace winrt::servo;
using namespace winrt::Windows;
using namespace winrt::Windows::Data::Json;
using namespace winrt::Windows::Foundation;
using namespace winrt::Windows::UI::Xaml;
using namespace winrt::Windows::UI::Xaml::Media;

static const hstring FXR_SCHEME = L"fxr";
static const hstring FXR_SCHEME_SLASH_SLASH = L"fxr://";
static const hstring FXRMIN_SCHEME = L"fxrmin";
static const hstring FXRMIN_SCHEME_SLASH_SLASH = L"fxrmin://";

struct BrowserPage : BrowserPageT<BrowserPage>, public DevtoolsDelegate {
public:
  BrowserPage();

  void OnForwardButtonClicked(IInspectable const &, RoutedEventArgs const &);
  void OnBackButtonClicked(IInspectable const &, RoutedEventArgs const &);
  void OnReloadButtonClicked(IInspectable const &, RoutedEventArgs const &);
  void OnStopButtonClicked(IInspectable const &, RoutedEventArgs const &);
  void OnHomeButtonClicked(IInspectable const &, RoutedEventArgs const &);
  void OnDevtoolsButtonClicked(IInspectable const &, RoutedEventArgs const &);
  void OnBookmarkClicked(IInspectable const &, RoutedEventArgs const &);
  void OnUpdateBookmarkButtonClicked(IInspectable const &,
                                     RoutedEventArgs const &) {
    UpdateBookmark();
  };
  void OnRemoveBookmarkButtonClicked(IInspectable const &,
                                     RoutedEventArgs const &) {
    RemoveBookmark();
  };
  void OnBookmarkEdited(IInspectable const &,
                        Input::KeyRoutedEventArgs const &);
  void OnJSInputEdited(IInspectable const &, Input::KeyRoutedEventArgs const &);
  void OnURLEdited(IInspectable const &, Input::KeyRoutedEventArgs const &);
  void OnSeeAllPrefClicked(IInspectable const &, RoutedEventArgs const &);
  void OnURLFocused(IInspectable const &);
  void
  OnURLKeyboardAccelerator(IInspectable const &,
                           Input::KeyboardAcceleratorInvokedEventArgs const &);
  void Shutdown();
  void LoadFXRURI(Uri uri);
  void SetArgs(hstring);
  void OnDismissCrashReport(IInspectable const &, RoutedEventArgs const &);
  void OnSubmitCrashReport(IInspectable const &, RoutedEventArgs const &);
  void OnMediaControlsPlayClicked(IInspectable const &,
                                  RoutedEventArgs const &);
  void OnMediaControlsPauseClicked(IInspectable const &,
                                   RoutedEventArgs const &);
  void OnPrefererenceSearchboxEdited(IInspectable const &,
                                     Input::KeyRoutedEventArgs const &);
  void OnDevtoolsMessage(DevtoolsMessageLevel, hstring, hstring);
  void ClearConsole();
  void OnDevtoolsDetached();
  Collections::IObservableVector<IInspectable> ConsoleLogs() { return mLogs; };
  Collections::IObservableVector<IInspectable> Bookmarks() {
    return mBookmarks.TemplateSource();
  };
  void RemoveBookmark();
  void UpdateBookmark();

private:
  void SetTransientMode(bool);
  void UpdatePref(ServoApp::Pref, Controls::Control);
  void CheckCrashReport();
  void BindServoEvents();
  void ShowToolbox();
  void HideToolbox();
  void BuildPrefList();
  void UpdateBookmarkPanel();
  void OnBookmarkDBChanged();
  DevtoolsStatus mDevtoolsStatus = DevtoolsStatus::Stopped;
  unsigned int mDevtoolsPort = 0;
  hstring mDevtoolsToken;
  bool mPanicking = false;
  std::unique_ptr<DevtoolsClient> mDevtoolsClient;
  Collections::IObservableVector<IInspectable> mLogs;
  std::map<hstring, hstring> mPromotedPrefs;
  std::optional<hstring> mCurrentUrl;
  std::optional<hstring> mCurrentTitle;
  servo::Bookmarks mBookmarks;
};

struct ConsoleLog : ConsoleLogT<ConsoleLog> {
public:
  ConsoleLog(Windows::UI::Color glyph, hstring g, hstring b, hstring s)
      : mGlyph(g), mSource(s), mBody(b) {
    mGlyphColor = UI::Xaml::Media::SolidColorBrush(glyph);
  };
  SolidColorBrush GlyphColor() { return mGlyphColor; };
  hstring Glyph() { return mGlyph; };
  hstring Source() { return mSource; };
  hstring Body() { return mBody; };

private:
  SolidColorBrush mGlyphColor;
  hstring mGlyph;
  hstring mSource;
  hstring mBody;
};

struct Bookmark : BookmarkT<Bookmark> {
public:
  Bookmark(hstring url, hstring name) : mName(name), mUrl(url){};
  hstring Name() { return mName; };
  hstring Url() { return mUrl; };

private:
  hstring mName;
  hstring mUrl;
};

} // namespace winrt::ServoApp::implementation

namespace winrt::ServoApp::factory_implementation {
struct BrowserPage : BrowserPageT<BrowserPage, implementation::BrowserPage> {};
struct ConsoleLog : ConsoleLogT<ConsoleLog, implementation::ConsoleLog> {};
struct Bookmark : BookmarkT<Bookmark, implementation::Bookmark> {};
} // namespace winrt::ServoApp::factory_implementation
