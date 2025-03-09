'use strict'

// Convert `manufacturerData` to an array of bluetooth.BluetoothManufacturerData
// defined in
// https://webbluetoothcg.github.io/web-bluetooth/#bluetooth-bidi-definitions.
function convertToBidiManufacturerData(manufacturerData) {
  const bidiManufacturerData = [];
  for (const key in manufacturerData) {
    bidiManufacturerData.push(
        {key: parseInt(key), data: btoa(manufacturerData[key].buffer)})
  }
  return bidiManufacturerData;
}

class FakeBluetooth {
  constructor() {
    this.fake_central_ = null;
  }

  // Returns a promise that resolves with a FakeCentral that clients can use
  // to simulate events that a device in the Central/Observer role would
  // receive as well as monitor the operations performed by the device in the
  // Central/Observer role.
  //
  // A "Central" object would allow its clients to receive advertising events
  // and initiate connections to peripherals i.e. operations of two roles
  // defined by the Bluetooth Spec: Observer and Central.
  // See Bluetooth 4.2 Vol 3 Part C 2.2.2 "Roles when Operating over an
  // LE Physical Transport".
  async simulateCentral({state}) {
    if (this.fake_central_) {
      throw 'simulateCentral() should only be called once';
    }

    await test_driver.bidi.bluetooth.simulate_adapter({state: state});
    this.fake_central_ = new FakeCentral();
    return this.fake_central_;
  }
}

// FakeCentral allows clients to simulate events that a device in the
// Central/Observer role would receive as well as monitor the operations
// performed by the device in the Central/Observer role.
class FakeCentral {
  constructor() {
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
    await test_driver.bidi.bluetooth.simulate_preconnected_peripheral({
      address: address,
      name: name,
      manufacturerData: convertToBidiManufacturerData(manufacturerData),
      knownServiceUuids: knownServiceUUIDs
    });

    return this.fetchOrCreatePeripheral_(address);
  }

  // Create a fake_peripheral object from the given address.
  fetchOrCreatePeripheral_(address) {
    let peripheral = this.peripherals_.get(address);
    if (peripheral === undefined) {
      peripheral = new FakePeripheral(address);
      this.peripherals_.set(address, peripheral);
    }
    return peripheral;
  }
}

class FakePeripheral {
  constructor(address) {
    this.address = address;
  }
}

function initializeBluetoothBidiResources() {
  navigator.bluetooth.test = new FakeBluetooth();
}
