/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

#pragma once

#include "pch.h"
#include "Bookmark.g.h"

namespace winrt::servo {

using namespace winrt::Windows::Foundation;

class Bookmarks {

public:
  Bookmarks();
  bool Contains(const hstring &url);
  hstring GetName(const hstring &url);
  void Set(hstring url, hstring title);
  void Delete(const hstring &url);
  const Collections::IObservableVector<IInspectable> &TemplateSource() {
    return db;
  };

private:
  IAsyncAction WriteSettings();
  void BuildIndex();
  void InvalidateDB();
  // Array of Bookmarks as defined in the IDL
  // An IObservableMap would be better, but this is not supported in XAML+winrt:
  // See https://github.com/microsoft/microsoft-ui-xaml/issues/1612
  Collections::IObservableVector<IInspectable> db;
  // ... so we have an additional map that link a url to the index in the array:
  std::map<hstring, int> mIndex;
};

} // namespace winrt::servo
