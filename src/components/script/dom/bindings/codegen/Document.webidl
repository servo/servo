/* -*- Mode: IDL; tab-width: 2; indent-tabs-mode: nil; c-basic-offset: 2 -*- */
/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this file,
 * You can obtain one at http://mozilla.org/MPL/2.0/.
 *
 * The origin of this IDL file is:
 * http://dom.spec.whatwg.org/#interface-document
 * http://www.whatwg.org/specs/web-apps/current-work/#the-document-object
 * http://dvcs.w3.org/hg/fullscreen/raw-file/tip/Overview.html#api
 * http://dvcs.w3.org/hg/pointerlock/raw-file/default/index.html#extensions-to-the-document-interface
 * http://dvcs.w3.org/hg/webperf/raw-file/tip/specs/PageVisibility/Overview.html#sec-document-interface
 * http://dev.w3.org/csswg/cssom/#extensions-to-the-document-interface
 * http://dev.w3.org/csswg/cssom-view/#extensions-to-the-document-interface
 *
 * http://mxr.mozilla.org/mozilla-central/source/dom/interfaces/core/nsIDOMDocument.idl
 */

/*interface StyleSheetList;
interface WindowProxy;
interface nsISupports;
interface URI;*/

enum VisibilityState { "hidden", "visible" };

/* http://dom.spec.whatwg.org/#interface-document */
[Constructor]
interface Document : Node {
  /*[Throws]
    readonly attribute DOMImplementation implementation;*/
  // readonly attribute DOMString URL;
  // readonly attribute DOMString documentURI;
  // readonly attribute DOMString compatMode;
  // readonly attribute DOMString characterSet;
  // readonly attribute DOMString contentType;

  //readonly attribute DocumentType? doctype;
  readonly attribute Element? documentElement;
  HTMLCollection getElementsByTagName(DOMString localName);
  HTMLCollection getElementsByTagNameNS(DOMString? namespace, DOMString localName);
  HTMLCollection getElementsByClassName(DOMString classNames);
  Element? getElementById(DOMString elementId);

  [Creator, Throws]
  Element createElement(DOMString localName);
  // [Creator, Throws]
  // Element createElementNS(DOMString? namespace, DOMString qualifiedName);
  [Creator]
  DocumentFragment createDocumentFragment();
  [Creator]
  Text createTextNode(DOMString data);
  [Creator]
  Comment createComment(DOMString data);
  /*[Creator, Throws]
  ProcessingInstruction createProcessingInstruction(DOMString target, DOMString data);*/

  /*[Throws]
  Node importNode(Node node, optional boolean deep = true);
  [Throws]
  Node adoptNode(Node node);*/

  [Creator, Throws]
  Event createEvent(DOMString interface_);

  /*[Creator, Throws]
    Range createRange();*/

  // NodeFilter.SHOW_ALL = 0xFFFFFFFF
  /*[Creator, Throws]
  NodeIterator createNodeIterator(Node root, optional unsigned long whatToShow = 0xFFFFFFFF, optional NodeFilter? filter = null);
  [Creator, Throws]
  TreeWalker createTreeWalker(Node root, optional unsigned long whatToShow = 0xFFFFFFFF, optional NodeFilter? filter = null);*/

  // NEW
  // No support for prepend/append yet
  // void prepend((Node or DOMString)... nodes);
  // void append((Node or DOMString)... nodes);

  // These are not in the spec, but leave them for now for backwards compat.
  // So sort of like Gecko extensions
  /*[Creator, Throws]
  CDATASection createCDATASection(DOMString data);
  [Creator, Throws]
  Attr createAttribute(DOMString name);
  [Creator, Throws]
  Attr createAttributeNS(DOMString? namespace, DOMString name);*/
  // readonly attribute DOMString? inputEncoding;
};

// http://www.whatwg.org/specs/web-apps/current-work/#the-document-object
partial interface Document {
  //[PutForwards=href, Unforgeable] readonly attribute Location? location;
  //(HTML only)         attribute DOMString domain;
  // readonly attribute DOMString referrer;
  //(HTML only)         attribute DOMString cookie;
  // readonly attribute DOMString lastModified;
  // readonly attribute DOMString readyState;

  // DOM tree accessors
  //(Not proxy yet)getter object (DOMString name);
           [SetterThrows]
           attribute DOMString title;
  //          attribute DOMString dir;
           attribute HTMLElement? body;
  readonly attribute HTMLHeadElement? head;
  //(HTML only)readonly attribute HTMLCollection images;
  //(HTML only)readonly attribute HTMLCollection embeds;
  //(HTML only)readonly attribute HTMLCollection plugins;
  //(HTML only)readonly attribute HTMLCollection links;
  //(HTML only)readonly attribute HTMLCollection forms;
  //(HTML only)readonly attribute HTMLCollection scripts;
  /*NodeList*/ HTMLCollection getElementsByName(DOMString elementName); ////(HTML only)
  //(HTML only)NodeList getItems(optional DOMString typeNames); // microdata
  //(Not implemented)readonly attribute DOMElementMap cssElementMap;

  // dynamic markup insertion
  //(HTML only)Document open(optional DOMString type, optional DOMString replace);
  //(HTML only)WindowProxy open(DOMString url, DOMString name, DOMString features, optional boolean replace);
  //(HTML only)void close();
  //(HTML only)void write(DOMString... text);
  //(HTML only)void writeln(DOMString... text);

  // user interaction
  // readonly attribute WindowProxy? defaultView;
  // readonly attribute Element? activeElement;
  // [Throws]
  // boolean hasFocus();
  //(HTML only)         attribute DOMString designMode;
  //(HTML only)boolean execCommand(DOMString commandId);
  //(HTML only)boolean execCommand(DOMString commandId, boolean showUI);
  //(HTML only)boolean execCommand(DOMString commandId, boolean showUI, DOMString value);
  //(HTML only)boolean queryCommandEnabled(DOMString commandId);
  //(HTML only)boolean queryCommandIndeterm(DOMString commandId);
  //(HTML only)boolean queryCommandState(DOMString commandId);
  //(HTML only)boolean queryCommandSupported(DOMString commandId);
  //(HTML only)DOMString queryCommandValue(DOMString commandId);
  //(Not implemented)readonly attribute HTMLCollection commands;

  // special event handler IDL attributes that only apply to Document objects
  //[LenientThis, SetterThrows] attribute EventHandler onreadystatechange;

  // Gecko extensions?
  /*[LenientThis, SetterThrows] attribute EventHandler onmouseenter;
  [LenientThis, SetterThrows] attribute EventHandler onmouseleave;
  [SetterThrows] attribute EventHandler onwheel;
  [SetterThrows] attribute EventHandler oncopy;
  [SetterThrows] attribute EventHandler oncut;
  [SetterThrows] attribute EventHandler onpaste;
  [SetterThrows] attribute EventHandler onbeforescriptexecute;
  [SetterThrows] attribute EventHandler onafterscriptexecute;*/
  /**
   * True if this document is synthetic : stand alone image, video, audio file,
   * etc.
   */
  //[ChromeOnly] readonly attribute boolean mozSyntheticDocument;
  /**
   * Returns the script element whose script is currently being processed.
   *
   * @see <https://developer.mozilla.org/en/DOM/document.currentScript>
   */
  // readonly attribute Element? currentScript;
  /**
   * Release the current mouse capture if it is on an element within this
   * document.
   *
   * @see <https://developer.mozilla.org/en/DOM/document.releaseCapture>
   */
  // void releaseCapture();
  /**
   * Use the given DOM element as the source image of target |-moz-element()|.
   *
   * This function introduces a new special ID (called "image element ID"),
   * which is only used by |-moz-element()|, and associates it with the given
   * DOM element.  Image elements ID's have the higher precedence than general
   * HTML id's, so if |document.mozSetImageElement(<id>, <element>)| is called,
   * |-moz-element(#<id>)| uses |<element>| as the source image even if there
   * is another element with id attribute = |<id>|.  To unregister an image
   * element ID |<id>|, call |document.mozSetImageElement(<id>, null)|.
   *
   * Example:
   * <script>
   *   canvas = document.createElement("canvas");
   *   canvas.setAttribute("width", 100);
   *   canvas.setAttribute("height", 100);
   *   // draw to canvas
   *   document.mozSetImageElement("canvasbg", canvas);
   * </script>
   * <div style="background-image: -moz-element(#canvasbg);"></div>
   *
   * @param aImageElementId an image element ID to associate with
   * |aImageElement|
   * @param aImageElement a DOM element to be used as the source image of
   * |-moz-element(#aImageElementId)|. If this is null, the function will
   * unregister the image element ID |aImageElementId|.
   *
   * @see <https://developer.mozilla.org/en/DOM/document.mozSetImageElement>
   */
  /*void mozSetImageElement(DOMString aImageElementId,
    Element? aImageElement);*/

  /*[ChromeOnly]
    readonly attribute URI? documentURIObject;*/

};

// http://dvcs.w3.org/hg/fullscreen/raw-file/tip/Overview.html#api
partial interface Document {
  // Note: Per spec the 'S' in these two is lowercase, but the "Moz"
  // versions hve it uppercase.
  // readonly attribute boolean mozFullScreenEnabled;
  // [Throws]
  // readonly attribute Element? mozFullScreenElement;

  //(Renamed?)void exitFullscreen();

  // Gecko-specific fullscreen bits
  /*readonly attribute boolean mozFullScreen;
    void mozCancelFullScreen();*/
};

// http://dvcs.w3.org/hg/pointerlock/raw-file/default/index.html#extensions-to-the-document-interface
partial interface Document {
    // readonly attribute Element? mozPointerLockElement;
    // void mozExitPointerLock ();
};

//http://dvcs.w3.org/hg/webcomponents/raw-file/tip/spec/custom/index.html#dfn-document-register
/*partial interface Document {
    [Throws, Pref="dom.webcomponents.enabled"]
    object register(DOMString name, optional ElementRegistrationOptions options);
    };*/

// http://dvcs.w3.org/hg/webperf/raw-file/tip/specs/PageVisibility/Overview.html#sec-document-interface
partial interface Document {
  // readonly attribute boolean hidden;
  // readonly attribute boolean mozHidden;
  // readonly attribute VisibilityState visibilityState;
  // readonly attribute VisibilityState mozVisibilityState;
};

// http://dev.w3.org/csswg/cssom/#extensions-to-the-document-interface
partial interface Document {
  /*[Constant]
    readonly attribute StyleSheetList styleSheets;*/
    // attribute DOMString? selectedStyleSheetSet;
    // readonly attribute DOMString? lastStyleSheetSet;
    // readonly attribute DOMString? preferredStyleSheetSet;
  /*[Constant]
    readonly attribute DOMStringList styleSheetSets;*/
    // void enableStyleSheetsForSet (DOMString? name);
};

// http://dev.w3.org/csswg/cssom-view/#extensions-to-the-document-interface
partial interface Document {
    // Element? elementFromPoint (float x, float y);

  //CaretPosition? caretPositionFromPoint (float x, float y);
};

// http://dvcs.w3.org/hg/undomanager/raw-file/tip/undomanager.html
/*partial interface Document {
    [Pref="dom.undo_manager.enabled"]
    readonly attribute UndoManager? undoManager;
    };*/

// http://dev.w3.org/2006/webapi/selectors-api2/#interface-definitions
partial interface Document {
  // [Throws]
  // Element?  querySelector(DOMString selectors);
  /*[Throws]
    NodeList  querySelectorAll(DOMString selectors);*/

  //(Not implemented)Element?  find(DOMString selectors, optional (Element or sequence<Node>)? refNodes);
  //(Not implemented)NodeList  findAll(DOMString selectors, optional (Element or sequence<Node>)? refNodes);
};

//  Mozilla extensions of various sorts
/*partial interface Document {
  // nsIDOMDocumentXBL.  Wish we could make these [ChromeOnly], but
  // that would likely break bindings running with the page principal.
  NodeList? getAnonymousNodes(Element elt);
  Element? getAnonymousElementByAttribute(Element elt, DOMString attrName,
                                          DOMString attrValue);
  Element? getBindingParent(Node node);
  [Throws]
  void loadBindingDocument(DOMString documentURL);

  // nsIDOMDocumentTouch
  // XXXbz I can't find the sane spec for this stuff, so just cribbing
  // from our xpidl for now.
  [Creator, Func="nsGenericHTMLElement::TouchEventsEnabled"]
  Touch createTouch(optional Window? view = null,
                    optional EventTarget? target = null,
                    optional long identifier = 0,
                    optional long pageX = 0,
                    optional long pageY = 0,
                    optional long screenX = 0,
                    optional long screenY = 0,
                    optional long clientX = 0,
                    optional long clientY = 0,
                    optional long radiusX = 0,
                    optional long radiusY = 0,
                    optional float rotationAngle = 0,
                    optional float force = 0);
  // XXXbz a hack to get around the fact that we don't support variadics as
  // distinguishing arguments yet.  Once this hack is removed. we can also
  // remove the corresponding overload on nsIDocument, since Touch... and
  // sequence<Touch> look the same in the C++.
  [Creator, Func="nsGenericHTMLElement::TouchEventsEnabled"]
  TouchList createTouchList(Touch touch, Touch... touches);
  // XXXbz and another hack for the fact that we can't usefully have optional
  // distinguishing arguments but need a working zero-arg form of
  // createTouchList().
  [Creator, Func="nsGenericHTMLElement::TouchEventsEnabled"]
  TouchList createTouchList();
  [Creator, Func="nsGenericHTMLElement::TouchEventsEnabled"]
  TouchList createTouchList(sequence<Touch> touches);

  [ChromeOnly]
  attribute boolean styleSheetChangeEventsEnabled;

  [ChromeOnly, Throws]
  void obsoleteSheet(URI sheetURI);
  [ChromeOnly, Throws]
  void obsoleteSheet(DOMString sheetURI);
};

// Extension to give chrome JS the ability to determine when a document was
// created to satisfy an iframe with srcdoc attribute.
partial interface Document {
  [ChromeOnly] readonly attribute boolean isSrcdocDocument;
};

Document implements XPathEvaluator;
Document implements GlobalEventHandlers;
Document implements NodeEventHandlers;
Document implements TouchEventHandlers;
Document implements ParentNode;*/
