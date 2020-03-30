/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

// https://jeenalee.github.io/doge-standard/

typedef sequence<ByteString> DogeInit;


[Exposed=(Window,Worker)]
interface Doge {
  [Throws] constructor(optional DogeInit init);
  void append(ByteString word);
  [Throws] ByteString random();
  [Throws] void Remove(ByteString word);
};
