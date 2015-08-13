/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

// https://html.spec.whatwg.org/multipage/#the-websocket-interface

enum BinaryType { "blob", "arraybuffer" };

[Constructor(DOMString url, optional /*(*/DOMString /*or DOMString[])*/ protocols)]
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
    //readonly attribute DOMString extensions;
    //readonly attribute DOMString protocol;
    [Throws] void close([Clamp] optional unsigned short code, optional USVString reason);

    //messaging
    attribute EventHandler onmessage;
    attribute BinaryType binaryType;
    [Throws] void send(optional USVString data);
    //void send(Blob data);
    //void send(ArrayBuffer data);
    //void send(ArrayBufferView data);

};
