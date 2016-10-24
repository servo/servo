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
pub enum BluetoothRequest {
    RequestDevice(RequestDeviceoptions, IpcSender<BluetoothResponse>),
    GATTServerConnect(String, IpcSender<BluetoothResponse>),
    GATTServerDisconnect(String, IpcSender<BluetoothResult<bool>>),
    GetPrimaryService(String, String, IpcSender<BluetoothResponse>),
    GetPrimaryServices(String, Option<String>, IpcSender<BluetoothResponse>),
    GetIncludedService(String, String, IpcSender<BluetoothResponse>),
    GetIncludedServices(String, Option<String>, IpcSender<BluetoothResponse>),
    GetCharacteristic(String, String, IpcSender<BluetoothResponse>),
    GetCharacteristics(String, Option<String>, IpcSender<BluetoothResponse>),
    GetDescriptor(String, String, IpcSender<BluetoothResponse>),
    GetDescriptors(String, Option<String>, IpcSender<BluetoothResponse>),
    ReadValue(String, IpcSender<BluetoothResponse>),
    WriteValue(String, Vec<u8>, IpcSender<BluetoothResponse>),
    Test(String, IpcSender<BluetoothResult<()>>),
    Exit,
}

#[derive(Deserialize, Serialize)]
pub enum BluetoothResponse {
    RequestDevice(BluetoothResult<BluetoothDeviceMsg>),
    GATTServerConnect(BluetoothResult<bool>),
    //GATTServerDisconnect(bool),
    GetPrimaryService(BluetoothResult<BluetoothServiceMsg>),
    GetPrimaryServices(BluetoothResult<BluetoothServicesMsg>),
    GetIncludedService(BluetoothResult<BluetoothServiceMsg>),
    GetIncludedServices(BluetoothResult<BluetoothServicesMsg>),
    GetCharacteristic(BluetoothResult<BluetoothCharacteristicMsg>),
    GetCharacteristics(BluetoothResult<BluetoothCharacteristicsMsg>),
    GetDescriptor(BluetoothResult<BluetoothDescriptorMsg>),
    GetDescriptors(BluetoothResult<BluetoothDescriptorsMsg>),
    ReadValue(BluetoothResult<Vec<u8>>),
    WriteValue(BluetoothResult<()>),
}

pub trait BluetoothResponseListener {
    fn response(&mut self, response: BluetoothResponse);
}

impl<T: BluetoothResponseListener> Action<T> for BluetoothResponse {
    /// Execute the default action on a provided listener.
    fn process(self, listener: &mut T) {
        listener.response(self)
    }
}
