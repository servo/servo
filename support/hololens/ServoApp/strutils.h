/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

#pragma once

template <typename... Args>
std::wstring format(const std::wstring &txt, Args... args) {
  size_t size = swprintf(nullptr, 0, txt.c_str(), args...) + 1;
  if (size <= 0) {
    throw std::runtime_error("Error during formatting.");
  }
  auto ptr = new wchar_t[size];
  swprintf(ptr, size, txt.c_str(), args...);
  auto wstr = std::wstring(ptr);
  delete[] ptr;
  return wstr;
}

template <typename... Args> void log(const std::wstring &txt, Args... args) {
  OutputDebugString((format(txt, args...) + L"\r\n").c_str());
}
