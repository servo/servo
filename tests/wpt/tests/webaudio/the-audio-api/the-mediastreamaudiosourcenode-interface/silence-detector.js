// AudioWorkletProcessor that detects silence and notifies the main thread on state change.
class SilenceDetector extends AudioWorkletProcessor {
  constructor() {
    super();
    // Assume silence and no input until the first buffer is processed.
    this.isSilent = true;
    this.hasInput = false;
  }

  process(inputs, outputs, parameters) {
    const input = inputs[0];
    const currentHasInput = input && input.length > 0;

    // Detect silence state (no input counts as silence)
    let isCurrentlySilent = true;
    if (currentHasInput) {
      const channel = input[0];
      for (let i = 0; i < channel.length; i++) {
        if (channel[i] !== 0) {
          isCurrentlySilent = false;
          break;
        }
      }
    }

    // Check if state changed
    const hasInputChanged = this.hasInput !== currentHasInput;
    const isSilentChanged = this.isSilent !== isCurrentlySilent;

    // Update state
    this.hasInput = currentHasInput;
    this.isSilent = isCurrentlySilent;

    // Send notifications
    if (hasInputChanged || isSilentChanged) {
      this.port.postMessage({
        type: "stateChanged",
        isSilent: this.isSilent,
        hasInput: this.hasInput,
        hasInputChanged: hasInputChanged,
        isSilentChanged: isSilentChanged,
      });
    }

    return currentHasInput;
  }
}

registerProcessor("silence-detector", SilenceDetector);
