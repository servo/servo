[Exposed=*]
interface AbortSignal : EventTarget {
  readonly attribute boolean aborted;
  readonly attribute any reason;
  undefined throwIfAborted();

  attribute EventHandler onabort;
};
