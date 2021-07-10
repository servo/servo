/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

#pragma once

#include "pch.h"

namespace winrt::servo {
using namespace winrt::Windows::Storage::Streams;
using namespace winrt::Windows::Data::Json;
using namespace winrt::Windows::Foundation;
using namespace winrt::Windows::Networking::Sockets;

class DevtoolsDelegate;

enum DevtoolsMessageLevel { Error, Warn, None };

class DevtoolsClient {

public:
  DevtoolsClient(hstring hostname, hstring port, hstring token,
                 DevtoolsDelegate &d)
      : mDelegate(d), mHostname(hostname), mToken(token), mPort(port){};

  ~DevtoolsClient() { Stop(); }
  void Run();
  void Stop();
  void Send(JsonObject);
  void Evaluate(hstring);

private:
  hstring mPort;
  hstring mToken;
  hstring mHostname;
  DevtoolsDelegate &mDelegate;
  std::optional<DataReader> mDataReader;
  std::optional<DataWriter> mDataWriter;
  std::optional<IAsyncAction> mReceiveOp;
  std::vector<JsonObject> mPendingObjects;
  IAsyncOperation<unsigned int> mReaderOp;
  bool mSending = false;
  bool mReceiving = false;
  void SendPendingObjects();
  IAsyncAction Loop();
  DevtoolsMessageLevel ParseLevel(JsonObject);
  hstring ParseSource(JsonObject);
  void HandleMessage(JsonObject);
  void HandlePageError(JsonObject);
  void HandleConsoleMessage(JsonObject);
  void HandleNonHandledMessage(JsonObject);
  void HandleEvaluationResult(JsonObject);
  std::optional<JsonValue> mConsoleActor;
};

class DevtoolsDelegate {
public:
  virtual void OnDevtoolsMessage(DevtoolsMessageLevel, hstring, hstring) = 0;
  virtual void ClearConsole() = 0;
  virtual void OnDevtoolsDetached() = 0;
};

} // namespace winrt::servo
