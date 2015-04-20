/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

enum BinaryType { "blob", "arraybuffer" };

[Constructor(DOMString url)]
interface WebSocket : EventTarget {
    readonly attribute DOMString url;
    //ready state
    const unsigned short CONNECTING = 0;
    const unsigned short OPEN = 1;
    const unsigned short CLOSING = 2;
    const unsigned short CLOSED = 3;
    readonly attribute unsigned short readyState;
    //readonly attribute unsigned long bufferedAmount;

    //networking
    attribute EventHandler onopen;
    attribute EventHandler onerror;
    attribute EventHandler onclose;
    [Throws]
    void Open();
    //readonly attribute DOMString extensions;
    //readonly attribute DOMString protocol;
    [Throws] void close(optional unsigned short code, optional DOMString reason);

    //messaging
    //attribute EventHandler onmessage;
    //attribute BinaryType binaryType;
    //[Throws] void send(USVString data);
    //void send(Blob data);
    //void send(ArrayBuffer data);
    //void send(ArrayBufferView data);

};
