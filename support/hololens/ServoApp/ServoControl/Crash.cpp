/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

#include "pch.h"
#include "Crash.h"
#include <chrono>
#include <ctime>
#include "Servo.h"

namespace winrt::servo {

using namespace Windows::Storage;

void WriteSection(StorageFile const &file, hstring section, hstring content) {
  hstring title{format(L"\r\n--- %s ---\r\n", section.c_str())};
  FileIO::AppendTextAsync(file, title).get();
  FileIO::AppendTextAsync(file, content).get();
}

void WriteCrashReport(hstring contentBacktrack, hstring contentUrl) {
  // Making all sync operations sync, as we are crashing.
  auto storageFolder = ApplicationData::Current().LocalFolder();
  auto fd = storageFolder
                .CreateFileAsync(L"crash-report.txt",
                                 CreationCollisionOption::ReplaceExisting)
                .get();
  FileIO::WriteTextAsync(fd, L"").get();

  // Stdout
  auto stdout_txt = storageFolder.GetFileAsync(L"stdout.txt").get();
  auto contentStdout = FileIO::ReadTextAsync(stdout_txt).get();

  // Crash time
  char cTime[70];
  auto crash_time = std::chrono::system_clock::now();
  auto now_c = std::chrono::system_clock::to_time_t(crash_time);
  std::tm now_tm;
  localtime_s(&now_tm, &now_c);
  strftime(cTime, sizeof cTime, "%FT%T%z", &now_tm);
  auto contentTime = char2hstring(cTime);

  // App + servo version
  auto pkg = winrt::Windows::ApplicationModel::Package::Current();
  auto v = pkg.Id().Version();
  auto servo_version = char2hstring(capi::servo_version());
  hstring contentVersion{format(L"%i.%i.%i.%i (%s)", v.Major, v.Minor, v.Build,
                                v.Revision, servo_version.c_str())};

  WriteSection(fd, L"CUSTOM MESSAGE",
               L"Feel free to add details here before reporting");
  WriteSection(fd, L"CURRENT URL (remove if sensitive)", contentUrl);
  WriteSection(fd, L"CRASH TIME", contentTime);
  WriteSection(fd, L"VERSION", contentVersion);
  WriteSection(fd, L"BACKTRACE", contentBacktrack);
  WriteSection(fd, L"STDOUT", contentStdout);
  FileIO::AppendTextAsync(fd, L"\r\n").get();
}

} // namespace winrt::servo
