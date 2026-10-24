[Exposed=Window]
interface MediaRecorder : EventTarget {
  [Throws] constructor(MediaStream stream, optional MediaRecorderOptions options = {});
  readonly attribute MediaStream stream;
  readonly attribute DOMString mimeType;
  readonly attribute RecordingState state;
  attribute EventHandler onstart;
  attribute EventHandler onstop;
  attribute EventHandler ondataavailable;
  attribute EventHandler onpause;
  attribute EventHandler onresume;
  attribute EventHandler onerror;
  readonly attribute unsigned long videoBitsPerSecond;
  readonly attribute unsigned long audioBitsPerSecond;
  readonly attribute BitrateMode audioBitrateMode;

  [Throws] undefined start(optional unsigned long timeslice);
  undefined stop();
  [Throws] undefined pause();
  [Throws] undefined resume();
  [Throws] undefined requestData();

  static boolean isTypeSupported(DOMString type);
};

dictionary MediaRecorderOptions {
  DOMString mimeType = "";
  unsigned long audioBitsPerSecond;
  unsigned long videoBitsPerSecond;
  unsigned long bitsPerSecond;
  BitrateMode audioBitrateMode = "variable";
  DOMHighResTimeStamp videoKeyFrameIntervalDuration;
  unsigned long videoKeyFrameIntervalCount;
};

enum BitrateMode {
  "constant",
  "variable"
};

enum RecordingState {
  "inactive",
  "recording",
  "paused"
};

[Exposed=Window]
interface BlobEvent : Event {
  constructor(DOMString type, BlobEventInit eventInitDict);
  [SameObject] readonly attribute Blob data;
  readonly attribute DOMHighResTimeStamp timecode;
};

dictionary BlobEventInit : EventInit {
  required Blob data;
  DOMHighResTimeStamp timecode;
};

