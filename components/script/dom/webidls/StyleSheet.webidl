interface StyleSheet {
  readonly attribute DOMString type_;
  readonly attribute DOMString? href;
  readonly attribute (Element or ProcessingInstruction)? ownerNode;
  readonly attribute DOMString? title;
  attribute boolean disabled;
};
