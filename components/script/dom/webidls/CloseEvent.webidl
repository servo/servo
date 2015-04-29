[Constructor(DOMString type, optional CloseEventInit eventInitDict)/*, Exposed=(Window,Worker)*/]
interface CloseEvent : Event {
  readonly attribute boolean wasClean;
  readonly attribute unsigned short code;
  readonly attribute DOMString reason;
};

dictionary CloseEventInit : EventInit {
  boolean wasClean;
  unsigned short code;
  DOMString reason;
};
