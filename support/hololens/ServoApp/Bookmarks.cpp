/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

#include "pch.h"
#include "Bookmarks.h"

namespace winrt::servo {

using namespace Windows::Storage;
using namespace Windows::UI::Core;
using namespace Windows::Foundation;
using namespace Windows::Data::Json;
using namespace Windows::ApplicationModel::Core;

Bookmarks::Bookmarks() {
  db = winrt::single_threaded_observable_vector<IInspectable>();
  auto x = Concurrency::create_task([=] {
    auto storageFolder = ApplicationData::Current().LocalFolder();
    auto bm_file = storageFolder.GetFileAsync(L"bookmarks.json").get();
    bool file_exist =
        storageFolder.TryGetItemAsync(L"bookmarks.json").get() != nullptr;
    if (file_exist) {
      auto content = FileIO::ReadTextAsync(bm_file).get();
      JsonValue out = JsonValue::Parse(L"[]");
      if (!JsonValue::TryParse(content, out)) {
        return;
      }
      auto list = out.GetArray();
      std::vector<IInspectable> bookmarks;
      for (auto value : list) {
        auto obj = value.GetObject();
        auto name = obj.GetNamedString(L"name");
        auto url = obj.GetNamedString(L"url");
        bookmarks.push_back(box_value(ServoApp::Bookmark(url, name)));
      }
      auto dispatcher = CoreApplication::MainView().CoreWindow().Dispatcher();
      dispatcher.RunAsync(CoreDispatcherPriority::High, [=] {
        db.ReplaceAll(bookmarks);
        BuildIndex();
      });
    }
  });
}

bool Bookmarks::Contains(const hstring &url) { return mIndex.count(url) > 0; }

void Bookmarks::Set(hstring url, hstring title) {
  auto bm = box_value(ServoApp::Bookmark(url, title));
  if (Contains(url)) {
    auto index = mIndex.at(url);
    db.SetAt(index, bm);
  } else {
    db.Append(bm);
  }
  InvalidateDB();
}

hstring Bookmarks::GetName(const hstring &url) {
  auto index = mIndex.at(url);
  ServoApp::Bookmark bm = unbox_value<ServoApp::Bookmark>(db.GetAt(index));
  return bm.Name();
}

void Bookmarks::Delete(const hstring &url) {
  auto index = mIndex.at(url);
  db.RemoveAt(index);
  InvalidateDB();
}

void Bookmarks::BuildIndex() {
  mIndex.clear();
  int i = 0;
  for (auto bm : db) {
    auto url = unbox_value<ServoApp::Bookmark>(bm).Url();
    mIndex.insert_or_assign(url, i++);
  }
}

void Bookmarks::InvalidateDB() {
  BuildIndex();
  WriteSettings();
}

IAsyncAction Bookmarks::WriteSettings() {
  auto storageFolder = ApplicationData::Current().LocalFolder();
  auto file = co_await storageFolder.CreateFileAsync(
      L"bookmarks.json", CreationCollisionOption::ReplaceExisting);
  JsonArray list;
  for (auto boxed_bm : db) {
    auto bm = unbox_value<ServoApp::Bookmark>(boxed_bm);
    JsonObject bookmark;
    bookmark.Insert(L"name", JsonValue::CreateStringValue(bm.Name()));
    bookmark.Insert(L"url", JsonValue::CreateStringValue(bm.Url()));
    list.Append(bookmark);
  }
  FileIO::WriteTextAsync(file, list.Stringify());
}

} // namespace winrt::servo
