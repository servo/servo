console.log(`Creating worker...`);
try {
  const channels = new Map();

  function getChannel(label) {
    const dc = channels.get(label);
    if (!dc) throw new Error(`No datachannel found for label ${label}`);
    return dc;
  }

  function reportWorkerError(label, message) {
    self.postMessage({ type: 'workerError', label, result: message });
  }

  function wireEvents(dc) {
    const eventTypes = ['open', 'bufferedamountlow', 'error', 'closing', 'close', 'message'];
    for (const type of eventTypes) {
      dc.addEventListener(type, ({data, origin}) => {
        if (type == 'message') {
          // Events aren't transferable in general, have to reconstruct :(
          self.postMessage({type, label: dc.label, result: {data, origin}});
        } else {
          self.postMessage({type, label: dc.label});
        }
      });
    }
  }

  onmessage = ({data}) => {
    const { type, label, arg } = data;

    try {
      switch (type) {
        case 'init': {
          const channel = arg;
          // We do not put errors in an initResponse message; those are for
          // RTCDataChannel errors only.
          if (channels.has(channel.label)) {
            throw new Error('Duplicate RTCDataChannel label');
          }
          channels.set(channel.label, channel);
          wireEvents(channel);
          self.postMessage({
            type: 'initResponse',
            label: channel.label,
            result: {
              label: channel.label,
              ordered: channel.ordered,
              maxPacketLifeTime: channel.maxPacketLifeTime,
              maxRetransmits: channel.maxRetransmits,
              protocol: channel.protocol,
              negotiated: channel.negotiated,
              id: channel.id,
              readyState: channel.readyState,
              bufferedAmount: channel.bufferedAmount,
              binaryType: channel.binaryType,
              bufferedAmountLowThreshold: channel.bufferedAmountLowThreshold,
            },
          });
          break;
        }

        case 'send': {
          // try block is only for RTCDataChannel errors
          const channel = getChannel(label);
          try {
            channel.send(arg);
            self.postMessage({ type: 'sendResponse', label, result: undefined });
          } catch (e) {
            // Man it would be nice if we could do "const error = {...} = e"
            const error = { name: e.name, message: e.message };
            self.postMessage({ type: 'sendResponse', label, result: { error } });
          }
          break;
        }

        case 'close': {
          // RTCDataChannel.close() does not throw; any error here is ours
          getChannel(label).close();
          self.postMessage({ type: 'closeResponse', label, result: undefined });
          break;
        }

        case 'setBufferedAmountLowThreshold': {
          // This is fire-and-forget called by a setter, any error will be
          // dealt with by the code in _errorPromise.
          getChannel(label).bufferedAmountLowThreshold = arg;
          break;
        }

        case 'setBinaryType': {
          // This is fire-and-forget called by a setter
          getChannel(label).binaryType = arg;
          break;
        }

        case 'queryState': {
          const channel = getChannel(label);
          self.postMessage({
            type: 'queryStateResponse',
            label,
            result: {
              id: channel.id,
              readyState: channel.readyState,
              bufferedAmount: channel.bufferedAmount,
            },
          });
          break;
        }

        default:
          console.log(`Unknown type`);
          reportWorkerError(label, `Received unknown message type: ${type}`);
      }
    } catch (err) {
      console.log(`Unhandled error`);
      reportWorkerError(label, `In handling ${type} request, got ${err.message}`);
    }
  };

} catch (e) {
  console.log(`Creation failed! ${e.message}`);
}

