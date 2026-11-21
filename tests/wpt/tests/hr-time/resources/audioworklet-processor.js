const moduleEvalNow = typeof globalThis.performance?.now === 'function' ?
    performance.now() :
    undefined;

class HrTimeProcessor extends AudioWorkletProcessor {
  constructor(options) {
    super();
    this.ctorNow = typeof globalThis.performance?.now === 'function' ?
        performance.now() :
        undefined;
    this.reportProcess = Boolean(options?.processorOptions?.reportProcess);
    this.processTimestamps = [];
    this.port.onmessage = (event) => this.handleMessage(event.data);
  }

  handleMessage(data) {
    const {id, command} = data;
    try {
      switch (command) {
        case 'interface': {
          const perf1 = globalThis.performance;
          const perf2 = globalThis.performance;
          this.port.postMessage({
            id,
            typeofWorkletPerformance: typeof globalThis.WorkletPerformance,
            hasPerformanceInterface: 'Performance' in globalThis,
            typeofPerformance: typeof perf1,
            typeofUnqualifiedPerformance: typeof performance,
            isSameObject: perf1 === perf2,
            isInstanceOfWorkletPerformance:
                typeof globalThis.WorkletPerformance === 'function' &&
                perf1 instanceof globalThis.WorkletPerformance,
            prototypeParentIsObjectPrototype:
                typeof globalThis.WorkletPerformance === 'function' &&
                Object.getPrototypeOf(
                    globalThis.WorkletPerformance.prototype) ===
                    Object.prototype,
            isInstanceOfEventTarget:
                typeof globalThis.EventTarget === 'function' &&
                perf1 instanceof globalThis.EventTarget,
            hasAddEventListener: perf1 !== undefined && perf1 !== null &&
                'addEventListener' in perf1,
            hasTimeOrigin:
                perf1 !== undefined && perf1 !== null && 'timeOrigin' in perf1,
            hasToJSON:
                perf1 !== undefined && perf1 !== null && 'toJSON' in perf1,
            typeofNow: typeof perf1?.now,
            typeofNowReturnValue: typeof perf1?.now === 'function' ?
                typeof perf1.now() :
                undefined,
          });
          break;
        }
        case 'replaceable': {
          const desc =
              Object.getOwnPropertyDescriptor(globalThis, 'performance');
          const initialDesc = desc ? {
            typeofGet: typeof desc.get,
            typeofSet: typeof desc.set,
            enumerable: desc.enumerable,
            configurable: desc.configurable,
          } :
                                     null;

          let afterValue;
          let replacedDesc;
          try {
            globalThis.performance = 'replaced';
            afterValue = globalThis.performance;
            replacedDesc =
                Object.getOwnPropertyDescriptor(globalThis, 'performance');
          } finally {
            if (desc) {
              if (desc.configurable) {
                Object.defineProperty(globalThis, 'performance', desc);
              }
            } else {
              delete globalThis.performance;
            }
          }
          this.port.postMessage({
            id,
            initialDesc,
            afterValue,
            replacedDesc: replacedDesc ? {
              value: replacedDesc.value,
              writable: replacedDesc.writable,
              enumerable: replacedDesc.enumerable,
              configurable: replacedDesc.configurable,
            } :
                                         null,
          });
          break;
        }
        case 'monotonic': {
          const hasNow = typeof globalThis.performance?.now === 'function';
          const now1 = hasNow ? performance.now() : undefined;
          const now2 = hasNow ? performance.now() : undefined;
          this.port.postMessage({
            id,
            moduleEvalNow,
            ctorNow: this.ctorNow,
            now1,
            now2,
          });
          break;
        }
        case 'now-and-date': {
          const hrt = typeof globalThis.performance?.now === 'function' ?
              performance.now() :
              undefined;
          const date = Date.now();
          this.port.postMessage({id, hrt, date});
          break;
        }
        case 'now': {
          const now = typeof globalThis.performance?.now === 'function' ?
              performance.now() :
              undefined;
          this.port.postMessage({id, now});
          break;
        }
        default: {
          this.port.postMessage({id, error: `Unknown command: ${command}`});
          break;
        }
      }
    } catch (e) {
      this.port.postMessage({id, error: `${e.name}: ${e.message}`});
    }
  }

  process() {
    if (!this.reportProcess) {
      return false;
    }
    const now = typeof globalThis.performance?.now === 'function' ?
        performance.now() :
        undefined;
    this.processTimestamps.push(now);
    if (this.processTimestamps.length >= 2) {
      this.port.postMessage({
        type: 'process-complete',
        processTimestamps: this.processTimestamps,
      });
      return false;
    }
    return true;
  }
}

registerProcessor('hr-time-processor', HrTimeProcessor);
