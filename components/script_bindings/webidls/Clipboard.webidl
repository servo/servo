/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

// https://w3c.github.io/clipboard-apis

typedef sequence<ClipboardItem> ClipboardItems;

[SecureContext, Exposed=Window, Pref="dom_async_clipboard_enabled"]
interface Clipboard : EventTarget {
  // Promise<ClipboardItems> read();
  Promise<DOMString> readText();
  // Promise<undefined> write(ClipboardItems data);
  Promise<undefined> writeText(DOMString data);
};

typedef Promise<(DOMString or Blob)> ClipboardItemData;

[SecureContext, Exposed=Window, Pref="dom_async_clipboard_enabled"]
interface ClipboardItem {
  [Throws] constructor(record<DOMString, ClipboardItemData> items,
              optional ClipboardItemOptions options = {});

  readonly attribute PresentationStyle presentationStyle;
  readonly attribute /* FrozenArray<DOMString> */ any types;

  // Promise<Blob> getType(DOMString type);

  // static boolean supports(DOMString type);
};

enum PresentationStyle { "unspecified", "inline", "attachment" };

dictionary ClipboardItemOptions {
  PresentationStyle presentationStyle = "unspecified";
};
