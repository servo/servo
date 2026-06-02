/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

// https://html.spec.whatwg.org/multipage/#the-websocket-interface

enum BinaryType { "blob", "arraybuffer" };

[Exposed=(Window,Worker)]
interface WebSocket : EventTarget {
    [Throws] constructor(DOMString url, optional (DOMString or sequence<DOMString>) protocols);
    readonly attribute DOMString url;
    //ready state
    const unsigned short CONNECTING = 0;
    const unsigned short OPEN = 1;
    const unsigned short CLOSING = 2;
    const unsigned short CLOSED = 3;
    readonly attribute unsigned short readyState;
    readonly attribute unsigned long long bufferedAmount;

    //networking
    attribute EventHandler onopen;
    attribute EventHandler onerror;
    attribute EventHandler onclose;
    //readonly attribute DOMString extensions;
    readonly attribute DOMString protocol;
    [Throws] undefined close(optional [Clamp] unsigned short code, optional USVString reason);

    //messaging
    attribute EventHandler onmessage;
    attribute BinaryType binaryType;
    [Throws] undefined send(USVString data);
    [Throws] undefined send(Blob data);
    [Throws] undefined send(ArrayBuffer data);
    [Throws] undefined send(ArrayBufferView data);
};
