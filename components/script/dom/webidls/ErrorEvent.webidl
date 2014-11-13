[Constructor(DOMString type, optional ErrorEventInit eventInitDict)/*, Exposed=(Window,Worker)*/]
interface ErrorEvent : Event {
  readonly attribute DOMString message;
  readonly attribute DOMString filename;
  readonly attribute unsigned long lineno;
  readonly attribute unsigned long colno;
  readonly attribute any error;
};
 
dictionary ErrorEventInit : EventInit {
  DOMString message;
  DOMString filename;
  unsigned long lineno;
  unsigned long colno;
  any error;
};

partial interface ErrorEvent {
// Deprecated in DOM Level 3
void initErrorEvent (DOMString typeArg, boolean bubblesArg, boolean cancelableArg, DOMString messageArg, DOMString filenameArg, 
unsigned long linenoArg, unsigned long colnoArg, any errorArg);
};
