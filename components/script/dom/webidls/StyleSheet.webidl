interface StyleSheet {
  readonly attribute DOMString type_;
  readonly attribute DOMString? href;
  readonly attribute (Element or ProcessingInstruction)? ownerNode;
  readonly attribute StyleSheet? parentStyleSheet;
  readonly attribute DOMString? title;
  [SameObject, PutForwards=mediaText] readonly attribute MediaList media;
  attribute boolean disabled;
};
