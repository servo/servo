/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

// https://fetch.spec.whatwg.org/#headers-class

typedef (Headers or sequence<sequence<ByteString>> or record<DOMString, ByteString>) HeadersInit;

[Constructor(optional HeadersInit init),
 Exposed=(Window,Worker)]
interface Headers {
  [Throws]
  void append(ByteString name, ByteString value);
  [Throws]
  void delete(ByteString name);
  [Throws]
  ByteString? get(ByteString name);
  [Throws]
  boolean has(ByteString name);
  [Throws]
  void set(ByteString name, ByteString value);
  iterable<ByteString, ByteString>;
};
