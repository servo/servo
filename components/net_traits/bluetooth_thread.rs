/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
use ipc_channel::ipc::IpcSender;

#[derive(Deserialize, Serialize)]
pub enum BluetoothMethodMsg {
    RequestDevice(IpcSender<BluetoothObjectMsg>),
    GATTServerConnect(String, IpcSender<BluetoothObjectMsg>),
    GATTServerDisconnect(String, IpcSender<BluetoothObjectMsg>),
    GetPrimaryService(String, IpcSender<BluetoothObjectMsg>),
    GetCharacteristic(String, IpcSender<BluetoothObjectMsg>),
    GetDescriptor(String, IpcSender<BluetoothObjectMsg>),
    ReadValue(String, IpcSender<BluetoothObjectMsg>),
    WriteValue(String, Vec<u8>, IpcSender<BluetoothObjectMsg>),
    Exit,
}

#[derive(Deserialize, Serialize)]
pub enum BluetoothObjectMsg {
    BluetoothDevice {
        // Bluetooth Device properties
        id: String,
        name: Option<String>,
        device_class: Option<u32>,
        vendor_id_source: Option<String>,
        vendor_id: Option<u32>,
        product_id: Option<u32>,
        product_version: Option<u32>,
        // Advertisiong Data properties
        appearance: Option<u16>,
        tx_power: Option<i8>,
        rssi: Option<i8>
    },
    BluetoothServer {
        connected: bool
    },
    BluetoothService {
        uuid: String,
        is_primary: bool,
        instance_id: String
    },
    BluetoothCharacteristic {
        // Characteristic
        uuid: String,
        instance_id: String,
        // Characteristic properties
        broadcast: bool,
        read: bool,
        write_without_response: bool,
        write: bool,
        notify: bool,
        indicate: bool,
        authenticated_signed_writes: bool,
        reliable_write: bool,
        writable_auxiliaries: bool
    },
    BluetoothDescriptor {
        uuid: String,
        instance_id: String
    },
    BluetoothReadValue {
        value: Vec<u8>
    },
    BluetoothWriteValue,
    Error {
        error: String
    },
}
