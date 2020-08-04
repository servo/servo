/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

#include "pch.h"
#include "strutils.h"
#include "Client.h"

using namespace winrt::Windows::Data::Json;
using namespace winrt::Windows::Foundation;
using namespace winrt::Windows::Networking;
using namespace winrt::Windows::Storage::Streams;

namespace winrt::servo {

void DevtoolsClient::Stop() {
  if (mReceiving && mReceiveOp.has_value() &&
      mReceiveOp->Status() != AsyncStatus::Completed) {
    mReceiveOp->Cancel();
  }
}

void DevtoolsClient::Run() {
  if (mReceiving) {
    throw hresult_error(E_FAIL, L"Already running");
  }
  mReceiving = true;
  auto socket = Sockets::StreamSocket();
  auto hostname = HostName(mHostname);
  auto connecting = socket.ConnectAsync(hostname, mPort);
  connecting.Completed([=](const auto &, const auto &) {
    mDataReader = DataReader(socket.InputStream());
    mDataWriter = DataWriter(socket.OutputStream());

    JsonObject out;
    out.Insert(L"auth_token", JsonValue::CreateStringValue(mToken));
    Send(out);

    mReceiveOp = {Loop()};
    mReceiveOp->Completed([=](const auto &, const auto &) {
      mReceiveOp = {};
      mDataReader->DetachStream();
      mDataWriter->DetachStream();
      mReceiving = false;
      mDelegate.OnDevtoolsDetached();
    });
  });
}

void DevtoolsClient::Evaluate(hstring code) {
  if (!code.empty() && mConsoleActor.has_value()) {
    JsonObject out;
    out.Insert(L"to", *mConsoleActor);
    out.Insert(L"type", JsonValue::CreateStringValue(L"evaluateJSAsync"));
    out.Insert(L"text", JsonValue::CreateStringValue(code));
    Send(out);
  }
}

IAsyncAction DevtoolsClient::Loop() {
  auto cancellation = co_await winrt::get_cancellation_token();
  cancellation.callback([=] {
    if (mReaderOp.Status() != AsyncStatus::Completed) {
      mReaderOp.Cancel();
    }
  });

  while (!cancellation()) {
    unsigned int len = 0;
    while (!cancellation()) {
      mReaderOp = mDataReader->LoadAsync(1);
      co_await mReaderOp;
      hstring c = mDataReader->ReadString(1);
      if (c == L":")
        break;
      try {
        unsigned int digit = std::stoi(c.c_str());
        len = 10 * len + digit;
      } catch (...) {
        throw hresult_error(E_FAIL, L"Can't parse message header:" + c);
      }
      if (len >= 100000) {
        throw hresult_error(E_FAIL, L"Message length too long");
      }
    }
    if (cancellation()) {
      break;
    }
    hstring request = L"";
    mReaderOp = mDataReader->LoadAsync(len);
    auto bytesLoaded = co_await mReaderOp;
    request = request + mDataReader->ReadString(bytesLoaded);
    JsonObject json;
    if (!JsonObject::TryParse(request, json)) {
      throw hresult_error(E_FAIL, L"Can't parse message: " + request);
    }
    HandleMessage(json);
  }
}

void DevtoolsClient::HandleMessage(JsonObject obj) {
  // Basic devtools protocol implementation:
  // https://docs.firefox-dev.tools/backend/protocol.html

  if (obj.HasKey(L"from") && obj.GetNamedString(L"from") == L"root") {
    if (obj.HasKey(L"applicationType")) {
      // First message. Ask for the current tab
      JsonObject out;
      out.Insert(L"to", JsonValue::CreateStringValue(L"root"));
      out.Insert(L"type", JsonValue::CreateStringValue(L"getTab"));
      Send(out);
      return;
    } else if (obj.HasKey(L"tab")) {
      // Got the current tab.
      auto tab = obj.GetNamedObject(L"tab");
      JsonObject out;
      out.Insert(L"to", tab.GetNamedValue(L"actor"));
      out.Insert(L"type", JsonValue::CreateStringValue(L"getTarget"));
      Send(out);
      return;
    }
  } else if (obj.HasKey(L"resultID")) {
    // evaluateJSAsync response.
    if (obj.GetNamedString(L"type", L"") == L"evaluationResult") {
      HandleEvaluationResult(obj);
    }
    return;
  } else if (obj.HasKey(L"type")) { // Not from root
    if (obj.GetNamedString(L"type") == L"pageError") {
      // Got a page error
      HandlePageError(obj.GetNamedObject(L"pageError"));
      return;
    } else if (obj.GetNamedString(L"type") == L"consoleAPICall") {
      // console.* calls
      auto message = obj.GetNamedObject(L"message");
      HandleConsoleMessage(message);
      return;
    } else if (obj.GetNamedString(L"type") == L"tabAttached") {
      // Ignore
      return;
    } else if (obj.GetNamedString(L"type") == L"networkEvent") {
      // Ignore
      return;
    } else if (obj.GetNamedString(L"type") == L"tabNavigated") {
      if (obj.HasKey(L"state") && obj.GetNamedString(L"state") == L"stop") {
        mDelegate.ClearConsole();
      }
      return;
    } else if (obj.GetNamedString(L"type") == L"networkEventUpdate") {
      // FIXME: log if there is a non-200 HTTP response
      return;
    }
  } else if (obj.HasKey(L"frame")) {
    auto frame = obj.GetNamedObject(L"frame");
    // Attach to tab, and ask for cached messaged
    JsonObject msg1;
    mConsoleActor = frame.GetNamedValue(L"consoleActor");
    msg1.Insert(L"to", frame.GetNamedValue(L"actor"));
    msg1.Insert(L"type", JsonValue::CreateStringValue(L"attach"));
    Send(msg1);
    JsonObject msg2;
    msg2.Insert(L"to", *mConsoleActor);
    msg2.Insert(L"type", JsonValue::CreateStringValue(L"getCachedMessages"));
    JsonArray types;
    types.Append(JsonValue::CreateStringValue(L"PageError"));
    types.Append(JsonValue::CreateStringValue(L"ConsoleAPI"));
    msg2.Insert(L"messageTypes", types);
    Send(msg2);
    return;
  } else if (obj.HasKey(L"messages")) {
    // Response to getCachedMessages
    for (auto messageValue : obj.GetNamedArray(L"messages")) {
      auto message = messageValue.GetObject();
      if (message.GetNamedString(L"_type") == L"ConsoleAPI") {
        HandleConsoleMessage(message);
      } else if (message.GetNamedString(L"_type") == L"PageError") {
        HandlePageError(message);
      } else {
        HandleNonHandledMessage(message);
      }
    }
    return;
  }
  HandleNonHandledMessage(obj);
}

DevtoolsMessageLevel DevtoolsClient::ParseLevel(JsonObject message) {
  if (message.GetNamedBoolean(L"error", false)) {
    return DevtoolsMessageLevel::Error;
  }
  if (message.GetNamedBoolean(L"warning", false)) {
    return DevtoolsMessageLevel::Warn;
  }
  if (message.GetNamedBoolean(L"exception", false)) {
    return DevtoolsMessageLevel::Error;
  }
  auto level = message.GetNamedString(L"level", L"");
  if (level == L"warn") {
    return DevtoolsMessageLevel::Warn;
  } else if (level == L"error") {
    return DevtoolsMessageLevel::Error;
  }
  return DevtoolsMessageLevel::None;
}

hstring DevtoolsClient::ParseSource(JsonObject message) {
  auto source = message.GetNamedString(L"filename", L"<>");
  if (message.HasKey(L"lineNumber")) {
    source = source + L":" + to_hstring(message.GetNamedNumber(L"lineNumber"));
  }
  if (message.HasKey(L"columnNumber")) {
    source =
        source + L":" + to_hstring(message.GetNamedNumber(L"columnNumber"));
  }
  return source;
}

void DevtoolsClient::HandlePageError(JsonObject message) {
  auto source = ParseSource(message);
  auto body = message.GetNamedString(L"errorMessage", L"");
  auto level = ParseLevel(message);
  mDelegate.OnDevtoolsMessage(level, source, body);
}

void DevtoolsClient::HandleEvaluationResult(JsonObject message) {
  auto level = DevtoolsMessageLevel::None;
  hstring body = L"";
  if (message.HasKey(L"result")) {
    auto value = message.GetNamedValue(L"result");
    if (value.ValueType() == JsonValueType::Object) {
      auto type = value.GetObject().GetNamedString(L"type");
      if (type == L"undefined") {
        body = L"undefined";
      } else {
        body = L"<object>";
      }
    } else {
      body = value.Stringify();
    }
  } else if (message.GetNamedValue(L"exception").ValueType() !=
             JsonValueType::Null) {
    level = DevtoolsMessageLevel::Error;
    body = message.GetNamedString(L"exceptionMessage", L"");
  }
  mDelegate.OnDevtoolsMessage(level, L"", body);
}

void DevtoolsClient::HandleConsoleMessage(JsonObject message) {
  auto source = ParseSource(message);
  auto level = ParseLevel(message);
  hstring body = L"";
  for (auto arg : message.GetNamedArray(L"arguments")) {
    body = body + arg.Stringify();
  }
  mDelegate.OnDevtoolsMessage(level, source, body);
}

void DevtoolsClient::HandleNonHandledMessage(JsonObject message) {
  auto level = DevtoolsMessageLevel::Warn;
  auto body = L"Unhandled devtools message: " + message.Stringify();
  mDelegate.OnDevtoolsMessage(level, L"", body);
}

void DevtoolsClient::SendPendingObjects() {
  if (mPendingObjects.empty() || mSending) {
    return;
  }
  mSending = true;
  auto obj = mPendingObjects.front();
  mPendingObjects.erase(mPendingObjects.begin());
  hstring msg = obj.Stringify();
  hstring size = to_hstring(msg.size());
  hstring request = size + L":" + msg;
  mDataWriter->WriteString(request);
  mDataWriter->StoreAsync().Completed([=](const auto &, const auto &) {
    mDataWriter->FlushAsync().Completed([=](const auto &, const auto &) {
      mSending = false;
      SendPendingObjects();
    });
  });
}

void DevtoolsClient::Send(JsonObject obj) {
  mPendingObjects.push_back(obj);
  SendPendingObjects();
}

} // namespace winrt::servo
