[SecureContext]
partial interface Navigator {
  [SameObject] readonly attribute WakeLock wakeLock;
};

enum WakeLockType { "screen" };

[SecureContext, Exposed=(Window)]
interface WakeLock {
  Promise<WakeLockSentinel> request(optional WakeLockType type = "screen");
};

[SecureContext, Exposed=(Window)]
interface WakeLockSentinel : EventTarget {
  readonly attribute boolean released;
  readonly attribute WakeLockType type;

  // Promise<void> release();
  // attribute EventHandler onrelease;
};
