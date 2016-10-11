/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use Action;
use bluetooth_scanfilter::RequestDeviceoptions;
use ipc_channel::ipc::IpcSender;

#[derive(Deserialize, Serialize)]
pub enum BluetoothError {
    Type(String),
    Network,
    NotFound,
    NotSupported,
    Security,
}

#[derive(Deserialize, Serialize)]
pub struct BluetoothDeviceMsg {
    // Bluetooth Device properties
    pub id: String,
    pub name: Option<String>,
    // Advertisiong Data properties
    pub appearance: Option<u16>,
    pub tx_power: Option<i8>,
    pub rssi: Option<i8>,
}

#[derive(Deserialize, Serialize)]
pub struct BluetoothServiceMsg {
    pub uuid: String,
    pub is_primary: bool,
    pub instance_id: String,
}

#[derive(Deserialize, Serialize)]
pub struct BluetoothCharacteristicMsg {
    // Characteristic
    pub uuid: String,
    pub instance_id: String,
    // Characteristic properties
    pub broadcast: bool,
    pub read: bool,
    pub write_without_response: bool,
    pub write: bool,
    pub notify: bool,
    pub indicate: bool,
    pub authenticated_signed_writes: bool,
    pub reliable_write: bool,
    pub writable_auxiliaries: bool,
}

#[derive(Deserialize, Serialize)]
pub struct BluetoothDescriptorMsg {
    pub uuid: String,
    pub instance_id: String,
}

pub type BluetoothServicesMsg = Vec<BluetoothServiceMsg>;

pub type BluetoothCharacteristicsMsg = Vec<BluetoothCharacteristicMsg>;

pub type BluetoothDescriptorsMsg = Vec<BluetoothDescriptorMsg>;

pub type BluetoothResult<T> = Result<T, BluetoothError>;

#[derive(Deserialize, Serialize)]
pub enum BluetoothMethodMsg {
    RequestDevice(RequestDeviceoptions, IpcSender<BluetoothResponseMsg>),
    GATTServerConnect(String, IpcSender<BluetoothResponseMsg>),
    GATTServerDisconnect(String, IpcSender<BluetoothResult<bool>>),
    GetPrimaryService(String, String, IpcSender<BluetoothResponseMsg>),
    GetPrimaryServices(String, Option<String>, IpcSender<BluetoothResponseMsg>),
    GetIncludedService(String, String, IpcSender<BluetoothResponseMsg>),
    GetIncludedServices(String, Option<String>, IpcSender<BluetoothResponseMsg>),
    GetCharacteristic(String, String, IpcSender<BluetoothResponseMsg>),
    GetCharacteristics(String, Option<String>, IpcSender<BluetoothResponseMsg>),
    GetDescriptor(String, String, IpcSender<BluetoothResponseMsg>),
    GetDescriptors(String, Option<String>, IpcSender<BluetoothResponseMsg>),
    ReadValue(String, IpcSender<BluetoothResponseMsg>),
    WriteValue(String, Vec<u8>, IpcSender<BluetoothResponseMsg>),
    Test(String, IpcSender<BluetoothResult<()>>),
    Exit,
}

#[derive(Deserialize, Serialize)]
pub enum BluetoothResultMsg {
    RequestDevice(BluetoothDeviceMsg),
    GATTServerConnect(bool),
    GATTServerDisconnect(bool),
    GetPrimaryService(BluetoothServiceMsg),
    GetPrimaryServices(BluetoothServicesMsg),
    GetIncludedService(BluetoothServiceMsg),
    GetIncludedServices(BluetoothServicesMsg),
    GetCharacteristic(BluetoothCharacteristicMsg),
    GetCharacteristics(BluetoothCharacteristicsMsg),
    GetDescriptor(BluetoothDescriptorMsg),
    GetDescriptors(BluetoothDescriptorsMsg),
    ReadValue(Vec<u8>),
    WriteValue(()),
    Error(BluetoothError),
}

#[derive(Deserialize, Serialize)]
pub enum BluetoothResponseMsg {
    Response(BluetoothResultMsg),
}

pub trait BluetoothTaskTarget {
    fn response(&mut self, result: BluetoothResultMsg);
}

pub trait BluetoothResponseListener {
    fn response(&mut self, result: BluetoothResultMsg);
}

impl BluetoothTaskTarget for IpcSender<BluetoothResponseMsg> {
    fn response(&mut self, result: BluetoothResultMsg) {
        let _ = self.send(BluetoothResponseMsg::Response(result));
    }
}

impl<T: BluetoothResponseListener> Action<T> for BluetoothResponseMsg {
    /// Execute the default action on a provided listener.
    fn process(self, listener: &mut T) {
        match self {
            BluetoothResponseMsg::Response(result) => listener.response(result),
        }
    }
}
