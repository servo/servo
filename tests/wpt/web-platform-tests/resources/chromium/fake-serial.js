// Implementation of an UnderlyingSource to create a ReadableStream from a Mojo
// data pipe consumer handle.
class DataPipeSource {
  constructor(consumer) {
    this.consumer_ = consumer;
  }

  async pull(controller) {
    let chunk = new ArrayBuffer(64);
    let {result, numBytes} = this.consumer_.readData(chunk);
    if (result == Mojo.RESULT_OK) {
      controller.enqueue(new Uint8Array(chunk, 0, numBytes));
      return;
    } else if (result == Mojo.RESULT_FAILED_PRECONDITION) {
      controller.close();
      return;
    } else if (result == Mojo.RESULT_SHOULD_WAIT) {
      await this.readable();
      return this.pull(controller);
    }
  }

  cancel() {
    if (this.watcher_)
      this.watcher_.cancel();
    this.consumer_.close();
  }

  readable() {
    return new Promise((resolve) => {
      this.watcher_ =
          this.consumer_.watch({ readable: true, peerClosed: true }, () => {
            this.watcher_.cancel();
            this.watcher_ = undefined;
            resolve();
          });
    });
  }
}

// Implementation of an UnderlyingSink to create a WritableStream from a Mojo
// data pipe producer handle.
class DataPipeSink {
  constructor(producer) {
    this._producer = producer;
  }

  async write(chunk, controller) {
    while (true) {
      let {result, numBytes} = this._producer.writeData(chunk);
      if (result == Mojo.RESULT_OK) {
        if (numBytes == chunk.byteLength) {
          return;
        }
        chunk = chunk.slice(numBytes);
      } else if (result == Mojo.RESULT_FAILED_PRECONDITION) {
        throw new DOMException('The pipe is closed.', 'InvalidStateError');
      } else if (result == Mojo.RESULT_SHOULD_WAIT) {
        await this.writable();
      }
    }
  }

  close() {
    assert_equals(undefined, this._watcher);
    this._producer.close();
  }

  abort(reason) {
    if (this._watcher)
      this._watcher.cancel();
    this._producer.close();
  }

  writable() {
    return new Promise((resolve) => {
      this._watcher =
          this._producer.watch({ writable: true, peerClosed: true }, () => {
            this._watcher.cancel();
            this._watcher = undefined;
            resolve();
          });
    });
  }
}

// Implementation of blink.mojom.SerialPort.
class FakeSerialPort {
  constructor() {
    this.inputSignals_ = {
      dataCarrierDetect: false,
      clearToSend: false,
      ringIndicator: false,
      dataSetReady: false
    };
    this.inputSignalFailure_ = false;
    this.outputSignals_ = {
      dataTerminalReady: false,
      requestToSend: false,
      break: false
    };
    this.outputSignalFailure_ = false;
  }

  open(options, client) {
    if (this.binding_ !== undefined) {
      // Port already open.
      return null;
    }

    let portPtr = new device.mojom.SerialPortPtr();
    this.binding_ = new mojo.Binding(
        device.mojom.SerialPort, this, mojo.makeRequest(portPtr));
    this.binding_.setConnectionErrorHandler(() => {
      this.close();
    });

    this.options_ = options;
    this.client_ = client;
    // OS typically sets DTR on open.
    this.outputSignals_.dataTerminalReady = true;

    return portPtr;
  }

  write(data) {
    return this.writer_.write(data);
  }

  read() {
    return this.reader_.read();
  }

  // Reads from the port until at least |targetLength| is read or the stream is
  // closed. The data is returned as a combined Uint8Array.
  readWithLength(targetLength) {
    return readWithLength(this.reader_, targetLength);
  }

  simulateReadError(error) {
    this.writer_.close();
    this.writer_.releaseLock();
    this.writer_ = undefined;
    this.writable_ = undefined;
    this.client_.onReadError(error);
  }

  simulateParityError() {
    this.simulateReadError(device.mojom.SerialReceiveError.PARITY_ERROR);
  }

  simulateDisconnectOnRead() {
    this.simulateReadError(device.mojom.SerialReceiveError.DISCONNECTED);
  }

  simulateWriteError(error) {
    this.reader_.cancel();
    this.reader_ = undefined;
    this.readable_ = undefined;
    this.client_.onSendError(error);
  }

  simulateSystemErrorOnWrite() {
    this.simulateWriteError(device.mojom.SerialSendError.SYSTEM_ERROR);
  }

  simulateDisconnectOnWrite() {
    this.simulateWriteError(device.mojom.SerialSendError.DISCONNECTED);
  }

  simulateInputSignals(signals) {
    this.inputSignals_ = signals;
  }

  simulateInputSignalFailure(fail) {
    this.inputSignalFailure_ = fail;
  }

  get outputSignals() {
    return this.outputSignals_;
  }

  simulateOutputSignalFailure(fail) {
    this.outputSignalFailure_ = fail;
  }

  writable() {
    if (this.writable_)
      return Promise.resolve();

    if (!this.writablePromise_) {
      this.writablePromise_ = new Promise((resolve) => {
        this.writableResolver_ = resolve;
      });
    }

    return this.writablePromise_;
  }

  readable() {
    if (this.readable_)
      return Promise.resolve();

    if (!this.readablePromise_) {
      this.readablePromise_ = new Promise((resolve) => {
        this.readableResolver_ = resolve;
      });
    }

    return this.readablePromise_;
  }

  async startWriting(in_stream) {
    this.readable_ = new ReadableStream(new DataPipeSource(in_stream));
    this.reader_ = this.readable_.getReader();
    if (this.readableResolver_) {
      this.readableResolver_();
      this.readableResolver_ = undefined;
      this.readablePromise_ = undefined;
    }
  }

  async startReading(out_stream) {
    this.writable_ = new WritableStream(new DataPipeSink(out_stream));
    this.writer_ = this.writable_.getWriter();
    if (this.writableResolver_) {
      this.writableResolver_();
      this.writableResolver_ = undefined;
      this.writablePromise_ = undefined;
    }
  }

  async flush(mode) {
    switch (mode) {
      case device.mojom.SerialPortFlushMode.kReceive:
        this.writer_.abort();
        this.writer_.releaseLock();
        this.writer_ = undefined;
        this.writable_ = undefined;
        break;
      case device.mojom.SerialPortFlushMode.kTransmit:
        this.reader_.cancel();
        this.reader_ = undefined;
        this.readable_ = undefined;
        break;
    }
  }

  async drain() {
    await this.reader_.closed;
  }

  async getControlSignals() {
    if (this.inputSignalFailure_) {
      return {signals: null};
    }

    const signals = {
      dcd: this.inputSignals_.dataCarrierDetect,
      cts: this.inputSignals_.clearToSend,
      ri: this.inputSignals_.ringIndicator,
      dsr: this.inputSignals_.dataSetReady
    };
    return {signals};
  }

  async setControlSignals(signals) {
    if (this.outputSignalFailure_) {
      return {success: false};
    }

    if (signals.hasDtr) {
      this.outputSignals_.dataTerminalReady = signals.dtr;
    }
    if (signals.hasRts) {
      this.outputSignals_.requestToSend = signals.rts;
    }
    if (signals.hasBrk) {
      this.outputSignals_.break = signals.brk;
    }
    return { success: true };
  }

  async configurePort(options) {
    this.options_ = options;
    return { success: true };
  }

  async getPortInfo() {
    return {
      bitrate: this.options_.bitrate,
      data_bits: this.options_.data_bits,
      parity_bit: this.options_.parity_bit,
      stop_bits: this.options_.stop_bits,
      cts_flow_control: this.options_.has_cts_flow_control ?
          this.options_.cts_flow_control : false
    };
  }

  async close() {
    // OS typically clears DTR on close.
    this.outputSignals_.dataTerminalReady = false;
    if (this.writer_) {
      this.writer_.close();
      this.writer_.releaseLock();
      this.writer_ = undefined;
    }
    this.writable_ = undefined;

    if (this.binding_) {
      this.binding_.close();
      this.binding_ = undefined;
    }
    return {};
  }
}

// Implementation of blink.mojom.SerialService.
class FakeSerialService {
  constructor() {
    this.interceptor_ =
        new MojoInterfaceInterceptor(blink.mojom.SerialService.name);
    this.interceptor_.oninterfacerequest = e => this.bind(e.handle);
    this.bindingSet_ = new mojo.BindingSet(blink.mojom.SerialService);
    this.clients_ = [];
    this.nextToken_ = 0;
    this.reset();
  }

  start() {
    this.interceptor_.start();
  }

  stop() {
    this.interceptor_.stop();
  }

  reset() {
    this.ports_ = new Map();
    this.selectedPort_ = null;
  }

  addPort(info) {
    let portInfo = new blink.mojom.SerialPortInfo();
    if (info?.usbVendorId !== undefined) {
      portInfo.hasUsbVendorId = true;
      portInfo.usbVendorId = info.usbVendorId;
    }
    if (info?.usbProductId !== undefined) {
      portInfo.hasUsbProductId = true;
      portInfo.usbProductId = info.usbProductId;
    }

    let token = ++this.nextToken_;
    portInfo.token = new mojoBase.mojom.UnguessableToken();
    portInfo.token.high = 0;
    portInfo.token.low = token;

    let record = {
      portInfo: portInfo,
      fakePort: new FakeSerialPort(),
    };
    this.ports_.set(token, record);

    for (let client of this.clients_) {
      client.onPortAdded(portInfo);
    }

    return token;
  }

  removePort(token) {
    let record = this.ports_.get(token);
    if (record === undefined) {
      return;
    }

    this.ports_.delete(token);

    for (let client of this.clients_) {
      client.onPortRemoved(record.portInfo);
    }
  }

  setSelectedPort(token) {
    this.selectedPort_ = this.ports_.get(token);
  }

  getFakePort(token) {
    let record = this.ports_.get(token);
    if (record === undefined)
      return undefined;
    return record.fakePort;
  }

  bind(handle) {
    this.bindingSet_.addBinding(this, handle);
  }

  async setClient(client_remote) {
    this.clients_.push(client_remote);
  }

  async getPorts() {
    return {
      ports: Array.from(this.ports_, ([token, record]) => record.portInfo)
    };
  }

  async requestPort(filters) {
    if (this.selectedPort_)
      return { port: this.selectedPort_.portInfo };
    else
      return { port: null };
  }

  async openPort(token, options, client) {
    let record = this.ports_.get(token.low);
    if (record !== undefined) {
      return {port: record.fakePort.open(options, client)};
    } else {
      return {port: null};
    }
  }
}

fakeSerialService = new FakeSerialService();
