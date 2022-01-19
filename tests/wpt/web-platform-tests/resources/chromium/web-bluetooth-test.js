'use strict';

const content = {};
const bluetooth = {};
const MOJO_CHOOSER_EVENT_TYPE_MAP = {};

function toMojoCentralState(state) {
  switch (state) {
    case 'absent':
      return bluetooth.mojom.CentralState.ABSENT;
    case 'powered-off':
      return bluetooth.mojom.CentralState.POWERED_OFF;
    case 'powered-on':
      return bluetooth.mojom.CentralState.POWERED_ON;
    default:
      throw `Unsupported value ${state} for state.`;
  }
}

// Converts bluetooth.mojom.WriteType to a string. If |writeType| is
// invalid, this method will throw.
function writeTypeToString(writeType) {
  switch (writeType) {
    case bluetooth.mojom.WriteType.kNone:
      return 'none';
    case bluetooth.mojom.WriteType.kWriteDefaultDeprecated:
      return 'default-deprecated';
    case bluetooth.mojom.WriteType.kWriteWithResponse:
      return 'with-response';
    case bluetooth.mojom.WriteType.kWriteWithoutResponse:
      return 'without-response';
    default:
      throw `Unknown bluetooth.mojom.WriteType: ${writeType}`;
  }
}

// Canonicalizes UUIDs and converts them to Mojo UUIDs.
function canonicalizeAndConvertToMojoUUID(uuids) {
  let canonicalUUIDs = uuids.map(val => ({uuid: BluetoothUUID.getService(val)}));
  return canonicalUUIDs;
}

// Converts WebIDL a record<DOMString, BufferSource> to a map<K, array<uint8>> to
// use for Mojo, where the value for K is calculated using keyFn.
function convertToMojoMap(record, keyFn, isNumberKey = false) {
  let map = new Map();
  for (const [key, value] of Object.entries(record)) {
    let buffer = ArrayBuffer.isView(value) ? value.buffer : value;
    if (isNumberKey) {
      let numberKey = parseInt(key);
      if (Number.isNaN(numberKey))
        throw `Map key ${key} is not a number`;
      map.set(keyFn(numberKey), Array.from(new Uint8Array(buffer)));
      continue;
    }
    map.set(keyFn(key), Array.from(new Uint8Array(buffer)));
  }
  return map;
}

function ArrayToMojoCharacteristicProperties(arr) {
  const struct = {};
  arr.forEach(property => { struct[property] = true; });
  return struct;
}

class FakeBluetooth {
  constructor() {
    this.fake_bluetooth_ptr_ = new bluetooth.mojom.FakeBluetoothRemote();
    this.fake_bluetooth_ptr_.$.bindNewPipeAndPassReceiver().bindInBrowser('process');
    this.fake_central_ = null;
  }

  // Set it to indicate whether the platform supports BLE. For example,
  // Windows 7 is a platform that doesn't support Low Energy. On the other
  // hand Windows 10 is a platform that does support LE, even if there is no
  // Bluetooth radio present.
  async setLESupported(supported) {
    if (typeof supported !== 'boolean') throw 'Type Not Supported';
    await this.fake_bluetooth_ptr_.setLESupported(supported);
  }

  // Returns a promise that resolves with a FakeCentral that clients can use
  // to simulate events that a device in the Central/Observer role would
  // receive as well as monitor the operations performed by the device in the
  // Central/Observer role.
  // Calls sets LE as supported.
  //
  // A "Central" object would allow its clients to receive advertising events
  // and initiate connections to peripherals i.e. operations of two roles
  // defined by the Bluetooth Spec: Observer and Central.
  // See Bluetooth 4.2 Vol 3 Part C 2.2.2 "Roles when Operating over an
  // LE Physical Transport".
  async simulateCentral({state}) {
    if (this.fake_central_)
      throw 'simulateCentral() should only be called once';

    await this.setLESupported(true);

    let {fakeCentral: fake_central_ptr} =
      await this.fake_bluetooth_ptr_.simulateCentral(
        toMojoCentralState(state));
    this.fake_central_ = new FakeCentral(fake_central_ptr);
    return this.fake_central_;
  }

  // Returns true if there are no pending responses.
  async allResponsesConsumed() {
    let {consumed} = await this.fake_bluetooth_ptr_.allResponsesConsumed();
    return consumed;
  }

  // Returns a promise that resolves with a FakeChooser that clients can use to
  // simulate chooser events.
  async getManualChooser() {
    if (typeof this.fake_chooser_ === 'undefined') {
      this.fake_chooser_ = new FakeChooser();
    }
    return this.fake_chooser_;
  }
}

// FakeCentral allows clients to simulate events that a device in the
// Central/Observer role would receive as well as monitor the operations
// performed by the device in the Central/Observer role.
class FakeCentral {
  constructor(fake_central_ptr) {
    this.fake_central_ptr_ = fake_central_ptr;
    this.peripherals_ = new Map();
  }

  // Simulates a peripheral with |address|, |name|, |manufacturerData| and
  // |known_service_uuids| that has already been connected to the system. If the
  // peripheral existed already it updates its name, manufacturer data, and
  // known UUIDs. |known_service_uuids| should be an array of
  // BluetoothServiceUUIDs
  // https://webbluetoothcg.github.io/web-bluetooth/#typedefdef-bluetoothserviceuuid
  //
  // Platforms offer methods to retrieve devices that have already been
  // connected to the system or weren't connected through the UA e.g. a user
  // connected a peripheral through the system's settings. This method is
  // intended to simulate peripherals that those methods would return.
  async simulatePreconnectedPeripheral(
      {address, name, manufacturerData = {}, knownServiceUUIDs = []}) {
    await this.fake_central_ptr_.simulatePreconnectedPeripheral(
        address, name,
        convertToMojoMap(manufacturerData, Number, true /* isNumberKey */),
        canonicalizeAndConvertToMojoUUID(knownServiceUUIDs));

    return this.fetchOrCreatePeripheral_(address);
  }

  // Simulates an advertisement packet described by |scanResult| being received
  // from a device. If central is currently scanning, the device will appear on
  // the list of discovered devices.
  async simulateAdvertisementReceived(scanResult) {
    // Create a deep-copy to prevent the original |scanResult| from being
    // modified when the UUIDs, manufacturer, and service data are converted.
    let clonedScanResult = JSON.parse(JSON.stringify(scanResult));

    if ('uuids' in scanResult.scanRecord) {
      clonedScanResult.scanRecord.uuids =
          canonicalizeAndConvertToMojoUUID(scanResult.scanRecord.uuids);
    }

    // Convert the optional appearance and txPower fields to the corresponding
    // Mojo structures, since Mojo does not support optional interger values. If
    // the fields are undefined, set the hasValue field as false and value as 0.
    // Otherwise, set the hasValue field as true and value with the field value.
    const has_appearance = 'appearance' in scanResult.scanRecord;
    clonedScanResult.scanRecord.appearance = {
      hasValue: has_appearance,
      value: (has_appearance ? scanResult.scanRecord.appearance : 0)
    }

    const has_tx_power = 'txPower' in scanResult.scanRecord;
    clonedScanResult.scanRecord.txPower = {
      hasValue: has_tx_power,
      value: (has_tx_power ? scanResult.scanRecord.txPower : 0)
    }

    // Convert manufacturerData from a record<DOMString, BufferSource> into a
    // map<uint8, array<uint8>> for Mojo.
    if ('manufacturerData' in scanResult.scanRecord) {
      clonedScanResult.scanRecord.manufacturerData = convertToMojoMap(
          scanResult.scanRecord.manufacturerData, Number,
          true /* isNumberKey */);
    }

    // Convert serviceData from a record<DOMString, BufferSource> into a
    // map<string, array<uint8>> for Mojo.
    if ('serviceData' in scanResult.scanRecord) {
      clonedScanResult.scanRecord.serviceData.serviceData = convertToMojoMap(
          scanResult.scanRecord.serviceData, BluetoothUUID.getService,
          false /* isNumberKey */);
    }

    await this.fake_central_ptr_.simulateAdvertisementReceived(
        clonedScanResult);

    return this.fetchOrCreatePeripheral_(clonedScanResult.deviceAddress);
  }

  // Simulates a change in the central device described by |state|. For example,
  // setState('powered-off') can be used to simulate the central device powering
  // off.
  //
  // This method should be used for any central state changes after
  // simulateCentral() has been called to create a FakeCentral object.
  async setState(state) {
    await this.fake_central_ptr_.setState(toMojoCentralState(state));
  }

  // Create a fake_peripheral object from the given address.
  fetchOrCreatePeripheral_(address) {
    let peripheral = this.peripherals_.get(address);
    if (peripheral === undefined) {
      peripheral = new FakePeripheral(address, this.fake_central_ptr_);
      this.peripherals_.set(address, peripheral);
    }
    return peripheral;
  }
}

class FakePeripheral {
  constructor(address, fake_central_ptr) {
    this.address = address;
    this.fake_central_ptr_ = fake_central_ptr;
  }

  // Adds a fake GATT Service with |uuid| to be discovered when discovering
  // the peripheral's GATT Attributes. Returns a FakeRemoteGATTService
  // corresponding to this service. |uuid| should be a BluetoothServiceUUIDs
  // https://webbluetoothcg.github.io/web-bluetooth/#typedefdef-bluetoothserviceuuid
  async addFakeService({uuid}) {
    let {serviceId: service_id} = await this.fake_central_ptr_.addFakeService(
      this.address, {uuid: BluetoothUUID.getService(uuid)});

    if (service_id === null) throw 'addFakeService failed';

    return new FakeRemoteGATTService(
      service_id, this.address, this.fake_central_ptr_);
  }

  // Sets the next GATT Connection request response to |code|. |code| could be
  // an HCI Error Code from BT 4.2 Vol 2 Part D 1.3 List Of Error Codes or a
  // number outside that range returned by specific platforms e.g. Android
  // returns 0x101 to signal a GATT failure
  // https://developer.android.com/reference/android/bluetooth/BluetoothGatt.html#GATT_FAILURE
  async setNextGATTConnectionResponse({code}) {
    let {success} =
      await this.fake_central_ptr_.setNextGATTConnectionResponse(
        this.address, code);

    if (success !== true) throw 'setNextGATTConnectionResponse failed.';
  }

  // Sets the next GATT Discovery request response for peripheral with
  // |address| to |code|. |code| could be an HCI Error Code from
  // BT 4.2 Vol 2 Part D 1.3 List Of Error Codes or a number outside that
  // range returned by specific platforms e.g. Android returns 0x101 to signal
  // a GATT failure
  // https://developer.android.com/reference/android/bluetooth/BluetoothGatt.html#GATT_FAILURE
  //
  // The following procedures defined at BT 4.2 Vol 3 Part G Section 4.
  // "GATT Feature Requirements" are used to discover attributes of the
  // GATT Server:
  //  - Primary Service Discovery
  //  - Relationship Discovery
  //  - Characteristic Discovery
  //  - Characteristic Descriptor Discovery
  // This method aims to simulate the response once all of these procedures
  // have completed or if there was an error during any of them.
  async setNextGATTDiscoveryResponse({code}) {
    let {success} =
      await this.fake_central_ptr_.setNextGATTDiscoveryResponse(
        this.address, code);

    if (success !== true) throw 'setNextGATTDiscoveryResponse failed.';
  }

  // Simulates a GATT disconnection from the peripheral with |address|.
  async simulateGATTDisconnection() {
    let {success} =
      await this.fake_central_ptr_.simulateGATTDisconnection(this.address);

    if (success !== true) throw 'simulateGATTDisconnection failed.';
  }

  // Simulates an Indication from the peripheral's GATT `Service Changed`
  // Characteristic from BT 4.2 Vol 3 Part G 7.1. This Indication is signaled
  // when services, characteristics, or descriptors are changed, added, or
  // removed.
  //
  // The value for `Service Changed` is a range of attribute handles that have
  // changed. However, this testing specification works at an abstracted
  // level and does not expose setting attribute handles when adding
  // attributes. Consequently, this simulate method should include the full
  // range of all the peripheral's attribute handle values.
  async simulateGATTServicesChanged() {
    let {success} =
      await this.fake_central_ptr_.simulateGATTServicesChanged(this.address);

    if (success !== true) throw 'simulateGATTServicesChanged failed.';
  }
}

class FakeRemoteGATTService {
  constructor(service_id, peripheral_address, fake_central_ptr) {
    this.service_id_ = service_id;
    this.peripheral_address_ = peripheral_address;
    this.fake_central_ptr_ = fake_central_ptr;
  }

  // Adds a fake GATT Characteristic with |uuid| and |properties|
  // to this fake service. The characteristic will be found when discovering
  // the peripheral's GATT Attributes. Returns a FakeRemoteGATTCharacteristic
  // corresponding to the added characteristic.
  async addFakeCharacteristic({uuid, properties}) {
    let {characteristicId: characteristic_id} =
        await this.fake_central_ptr_.addFakeCharacteristic(
            {uuid: BluetoothUUID.getCharacteristic(uuid)},
            ArrayToMojoCharacteristicProperties(properties),
            this.service_id_,
            this.peripheral_address_);

    if (characteristic_id === null) throw 'addFakeCharacteristic failed';

    return new FakeRemoteGATTCharacteristic(
      characteristic_id, this.service_id_,
      this.peripheral_address_, this.fake_central_ptr_);
  }

  // Removes the fake GATT service from its fake peripheral.
  async remove() {
    let {success} =
        await this.fake_central_ptr_.removeFakeService(
            this.service_id_,
            this.peripheral_address_);

    if (!success) throw 'remove failed';
  }
}

class FakeRemoteGATTCharacteristic {
  constructor(characteristic_id, service_id, peripheral_address,
      fake_central_ptr) {
    this.ids_ = [characteristic_id, service_id, peripheral_address];
    this.descriptors_ = [];
    this.fake_central_ptr_ = fake_central_ptr;
  }

  // Adds a fake GATT Descriptor with |uuid| to be discovered when
  // discovering the peripheral's GATT Attributes. Returns a
  // FakeRemoteGATTDescriptor corresponding to this descriptor. |uuid| should
  // be a BluetoothDescriptorUUID
  // https://webbluetoothcg.github.io/web-bluetooth/#typedefdef-bluetoothdescriptoruuid
  async addFakeDescriptor({uuid}) {
    let {descriptorId: descriptor_id} =
        await this.fake_central_ptr_.addFakeDescriptor(
            {uuid: BluetoothUUID.getDescriptor(uuid)}, ...this.ids_);

    if (descriptor_id === null) throw 'addFakeDescriptor failed';

    let fake_descriptor = new FakeRemoteGATTDescriptor(
      descriptor_id, ...this.ids_, this.fake_central_ptr_);
    this.descriptors_.push(fake_descriptor);

    return fake_descriptor;
  }

  // Sets the next read response for characteristic to |code| and |value|.
  // |code| could be a GATT Error Response from
  // BT 4.2 Vol 3 Part F 3.4.1.1 Error Response or a number outside that range
  // returned by specific platforms e.g. Android returns 0x101 to signal a GATT
  // failure.
  // https://developer.android.com/reference/android/bluetooth/BluetoothGatt.html#GATT_FAILURE
  async setNextReadResponse(gatt_code, value=null) {
    if (gatt_code === 0 && value === null) {
      throw '|value| can\'t be null if read should success.';
    }
    if (gatt_code !== 0 && value !== null) {
      throw '|value| must be null if read should fail.';
    }

    let {success} =
      await this.fake_central_ptr_.setNextReadCharacteristicResponse(
        gatt_code, value, ...this.ids_);

    if (!success) throw 'setNextReadCharacteristicResponse failed';
  }

  // Sets the next write response for this characteristic to |code|. If
  // writing to a characteristic that only supports 'write_without_response'
  // the set response will be ignored.
  // |code| could be a GATT Error Response from
  // BT 4.2 Vol 3 Part F 3.4.1.1 Error Response or a number outside that range
  // returned by specific platforms e.g. Android returns 0x101 to signal a GATT
  // failure.
  async setNextWriteResponse(gatt_code) {
    let {success} =
      await this.fake_central_ptr_.setNextWriteCharacteristicResponse(
        gatt_code, ...this.ids_);

    if (!success) throw 'setNextWriteCharacteristicResponse failed';
  }

  // Sets the next subscribe to notifications response for characteristic with
  // |characteristic_id| in |service_id| and in |peripheral_address| to
  // |code|. |code| could be a GATT Error Response from BT 4.2 Vol 3 Part F
  // 3.4.1.1 Error Response or a number outside that range returned by
  // specific platforms e.g. Android returns 0x101 to signal a GATT failure.
  async setNextSubscribeToNotificationsResponse(gatt_code) {
    let {success} =
      await this.fake_central_ptr_.setNextSubscribeToNotificationsResponse(
        gatt_code, ...this.ids_);

    if (!success) throw 'setNextSubscribeToNotificationsResponse failed';
  }

  // Sets the next unsubscribe to notifications response for characteristic with
  // |characteristic_id| in |service_id| and in |peripheral_address| to
  // |code|. |code| could be a GATT Error Response from BT 4.2 Vol 3 Part F
  // 3.4.1.1 Error Response or a number outside that range returned by
  // specific platforms e.g. Android returns 0x101 to signal a GATT failure.
  async setNextUnsubscribeFromNotificationsResponse(gatt_code) {
    let {success} =
      await this.fake_central_ptr_.setNextUnsubscribeFromNotificationsResponse(
        gatt_code, ...this.ids_);

    if (!success) throw 'setNextUnsubscribeToNotificationsResponse failed';
  }

  // Returns true if notifications from the characteristic have been subscribed
  // to.
  async isNotifying() {
    let {success, isNotifying} =
        await this.fake_central_ptr_.isNotifying(...this.ids_);

    if (!success) throw 'isNotifying failed';

    return isNotifying;
  }

  // Gets the last successfully written value to the characteristic and its
  // write type. Write type is one of 'none', 'default-deprecated',
  // 'with-response', 'without-response'. Returns {lastValue: null,
  // lastWriteType: 'none'} if no value has yet been written to the
  // characteristic.
  async getLastWrittenValue() {
    let {success, value, writeType} =
        await this.fake_central_ptr_.getLastWrittenCharacteristicValue(
            ...this.ids_);

    if (!success) throw 'getLastWrittenCharacteristicValue failed';

    return {lastValue: value, lastWriteType: writeTypeToString(writeType)};
  }

  // Removes the fake GATT Characteristic from its fake service.
  async remove() {
    let {success} =
        await this.fake_central_ptr_.removeFakeCharacteristic(...this.ids_);

    if (!success) throw 'remove failed';
  }
}

class FakeRemoteGATTDescriptor {
  constructor(descriptor_id,
              characteristic_id,
              service_id,
              peripheral_address,
              fake_central_ptr) {
    this.ids_ = [
      descriptor_id, characteristic_id, service_id, peripheral_address];
    this.fake_central_ptr_ = fake_central_ptr;
  }

  // Sets the next read response for descriptor to |code| and |value|.
  // |code| could be a GATT Error Response from
  // BT 4.2 Vol 3 Part F 3.4.1.1 Error Response or a number outside that range
  // returned by specific platforms e.g. Android returns 0x101 to signal a GATT
  // failure.
  // https://developer.android.com/reference/android/bluetooth/BluetoothGatt.html#GATT_FAILURE
  async setNextReadResponse(gatt_code, value=null) {
    if (gatt_code === 0 && value === null) {
      throw '|value| cannot be null if read should succeed.';
    }
    if (gatt_code !== 0 && value !== null) {
      throw '|value| must be null if read should fail.';
    }

    let {success} =
      await this.fake_central_ptr_.setNextReadDescriptorResponse(
        gatt_code, value, ...this.ids_);

    if (!success) throw 'setNextReadDescriptorResponse failed';
  }

  // Sets the next write response for this descriptor to |code|.
  // |code| could be a GATT Error Response from
  // BT 4.2 Vol 3 Part F 3.4.1.1 Error Response or a number outside that range
  // returned by specific platforms e.g. Android returns 0x101 to signal a GATT
  // failure.
  async setNextWriteResponse(gatt_code) {
    let {success} =
      await this.fake_central_ptr_.setNextWriteDescriptorResponse(
        gatt_code, ...this.ids_);

    if (!success) throw 'setNextWriteDescriptorResponse failed';
  }

  // Gets the last successfully written value to the descriptor.
  // Returns null if no value has yet been written to the descriptor.
  async getLastWrittenValue() {
    let {success, value} =
      await this.fake_central_ptr_.getLastWrittenDescriptorValue(
          ...this.ids_);

    if (!success) throw 'getLastWrittenDescriptorValue failed';

    return value;
  }

  // Removes the fake GATT Descriptor from its fake characteristic.
  async remove() {
    let {success} =
        await this.fake_central_ptr_.removeFakeDescriptor(...this.ids_);

    if (!success) throw 'remove failed';
  }
}

// FakeChooser allows clients to simulate user actions on a Bluetooth chooser,
// and records the events produced by the Bluetooth chooser.
class FakeChooser {
  constructor() {
    let fakeBluetoothChooserFactoryRemote =
        new content.mojom.FakeBluetoothChooserFactoryRemote();
    fakeBluetoothChooserFactoryRemote.$.bindNewPipeAndPassReceiver().bindInBrowser('process');

    this.fake_bluetooth_chooser_ptr_ =
        new content.mojom.FakeBluetoothChooserRemote();
    this.fake_bluetooth_chooser_client_receiver_ =
        new content.mojom.FakeBluetoothChooserClientReceiver(this);
    fakeBluetoothChooserFactoryRemote.createFakeBluetoothChooser(
        this.fake_bluetooth_chooser_ptr_.$.bindNewPipeAndPassReceiver(),
        this.fake_bluetooth_chooser_client_receiver_.$.associateAndPassRemote());

    this.events_ = new Array();
    this.event_listener_ = null;
  }

  // If the chooser has received more events than |numOfEvents| this function
  // will reject the promise, else it will wait until |numOfEvents| events are
  // received before resolving with an array of |FakeBluetoothChooserEvent|
  // objects.
  async waitForEvents(numOfEvents) {
    return new Promise(resolve => {
      if (this.events_.length > numOfEvents) {
        throw `Asked for ${numOfEvents} event(s), but received ` +
            `${this.events_.length}.`;
      }

      this.event_listener_ = () => {
         if (this.events_.length === numOfEvents) {
          let result = Array.from(this.events_);
          this.event_listener_ = null;
          this.events_ = [];
          resolve(result);
        }
      };
      this.event_listener_();
    });
  }

  async selectPeripheral(peripheral) {
    if (!(peripheral instanceof FakePeripheral)) {
      throw '|peripheral| must be an instance of FakePeripheral';
    }
    await this.fake_bluetooth_chooser_ptr_.selectPeripheral(peripheral.address);
  }

  async cancel() {
    await this.fake_bluetooth_chooser_ptr_.cancel();
  }

  async rescan() {
    await this.fake_bluetooth_chooser_ptr_.rescan();
  }

  onEvent(chooserEvent) {
    chooserEvent.type = MOJO_CHOOSER_EVENT_TYPE_MAP[chooserEvent.type];
    this.events_.push(chooserEvent);
    if (this.event_listener_ !== null) {
      this.event_listener_();
    }
  }
}

async function initializeChromiumResources() {
  content.mojom = await import(
      '/gen/content/web_test/common/fake_bluetooth_chooser.mojom.m.js');
  bluetooth.mojom = await import(
      '/gen/device/bluetooth/public/mojom/test/fake_bluetooth.mojom.m.js');

  const map = MOJO_CHOOSER_EVENT_TYPE_MAP;
  const types = content.mojom.ChooserEventType;
  map[types.CHOOSER_OPENED] = 'chooser-opened';
  map[types.CHOOSER_CLOSED] = 'chooser-closed';
  map[types.ADAPTER_REMOVED] = 'adapter-removed';
  map[types.ADAPTER_DISABLED] = 'adapter-disabled';
  map[types.ADAPTER_ENABLED] = 'adapter-enabled';
  map[types.DISCOVERY_FAILED_TO_START] = 'discovery-failed-to-start';
  map[types.DISCOVERING] = 'discovering';
  map[types.DISCOVERY_IDLE] = 'discovery-idle';
  map[types.ADD_OR_UPDATE_DEVICE] = 'add-or-update-device';

  // If this line fails, it means that current environment does not support the
  // Web Bluetooth Test API.
  try {
    navigator.bluetooth.test = new FakeBluetooth();
  } catch {
    throw 'Web Bluetooth Test API is not implemented on this ' +
        'environment. See the bluetooth README at ' +
        'https://github.com/web-platform-tests/wpt/blob/master/bluetooth/README.md#web-bluetooth-testing';
  }
}
