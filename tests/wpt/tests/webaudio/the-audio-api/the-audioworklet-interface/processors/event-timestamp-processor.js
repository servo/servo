class EventTimeStampProcessor extends AudioWorkletProcessor {
  constructor() {
    super();
    const timeStamp = new Event('x').timeStamp;
    this.port.postMessage({type: typeof timeStamp, value: timeStamp});
  }

  process() {
    return false;
  }
}

registerProcessor('event-timestamp-processor', EventTimeStampProcessor);
