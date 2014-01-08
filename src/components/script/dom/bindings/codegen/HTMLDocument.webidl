/* -*- Mode: IDL; tab-width: 2; indent-tabs-mode: nil; c-basic-offset: 2 -*- */
/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this file,
 * You can obtain one at http://mozilla.org/MPL/2.0/.
 */

interface Selection;

[OverrideBuiltins]
interface HTMLDocument : Document {
  //          [Throws]
  //          attribute DOMString? domain;
  //          [Throws]
  //          attribute DOMString cookie;
  // DOM tree accessors
  // [Throws]
  // getter object (DOMString name);
  /*[SetterThrows]
    attribute HTMLElement? body;*/
  // readonly attribute HTMLHeadElement? head;
  readonly attribute HTMLCollection images;
  readonly attribute HTMLCollection embeds;
  readonly attribute HTMLCollection plugins;
  readonly attribute HTMLCollection links;
  readonly attribute HTMLCollection forms;
  readonly attribute HTMLCollection scripts;
  /*NodeList getElementsByName(DOMString elementName);
    NodeList getItems(optional DOMString typeNames = ""); // microdata*/

  // dynamic markup insertion
  /*[Throws]
  Document open(optional DOMString type = "text/html", optional DOMString replace = "");
  [Throws]
  WindowProxy open(DOMString url, DOMString name, DOMString features, optional boolean replace = false);*/
  // [Throws]
  // void close();
  /*[Throws]
  void write(DOMString... text);
  [Throws]
  void writeln(DOMString... text);*/

  //          [SetterThrows]
  //          attribute DOMString designMode;
  // [Throws]
  // boolean execCommand(DOMString commandId, optional boolean showUI = false,
  //                     optional DOMString value = "");
  // [Throws]
  // boolean queryCommandEnabled(DOMString commandId);
  // [Throws]
  // boolean queryCommandIndeterm(DOMString commandId);
  // [Throws]
  // boolean queryCommandState(DOMString commandId);
  // boolean queryCommandSupported(DOMString commandId);
  // [Throws]
  // DOMString queryCommandValue(DOMString commandId);

  // [TreatNullAs=EmptyString] attribute DOMString fgColor;
  // [TreatNullAs=EmptyString] attribute DOMString linkColor;
  // [TreatNullAs=EmptyString] attribute DOMString vlinkColor;
  // [TreatNullAs=EmptyString] attribute DOMString alinkColor;
  // [TreatNullAs=EmptyString] attribute DOMString bgColor;

  readonly attribute HTMLCollection anchors;
  readonly attribute HTMLCollection applets;

  // void clear();

  // [Throws]
  // readonly attribute object all;

  // https://dvcs.w3.org/hg/editing/raw-file/tip/editing.html#selections
  /*[Throws]
    Selection getSelection();*/
};
