/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use std::collections::HashMap;
use std::error::Error;
use std::sync::Arc;

#[cfg(all(target_os = "android", feature = "native-bluetooth"))]
use blurdroid::bluetooth_device::Device as BluetoothDeviceAndroid;
#[cfg(all(target_os = "android", feature = "native-bluetooth"))]
use blurdroid::bluetooth_discovery_session::DiscoverySession as BluetoothDiscoverySessionAndroid;
#[cfg(all(target_os = "android", feature = "native-bluetooth"))]
use blurdroid::bluetooth_gatt_characteristic::Characteristic as BluetoothGATTCharacteristicAndroid;
#[cfg(all(target_os = "android", feature = "native-bluetooth"))]
use blurdroid::bluetooth_gatt_descriptor::Descriptor as BluetoothGATTDescriptorAndroid;
#[cfg(all(target_os = "android", feature = "native-bluetooth"))]
use blurdroid::bluetooth_gatt_service::Service as BluetoothGATTServiceAndroid;
#[cfg(all(target_os = "macos", feature = "native-bluetooth"))]
use blurmac::BluetoothDevice as BluetoothDeviceMac;
#[cfg(all(target_os = "macos", feature = "native-bluetooth"))]
use blurmac::BluetoothDiscoverySession as BluetoothDiscoverySessionMac;
#[cfg(all(target_os = "macos", feature = "native-bluetooth"))]
use blurmac::BluetoothGATTCharacteristic as BluetoothGATTCharacteristicMac;
#[cfg(all(target_os = "macos", feature = "native-bluetooth"))]
use blurmac::BluetoothGATTDescriptor as BluetoothGATTDescriptorMac;
#[cfg(all(target_os = "macos", feature = "native-bluetooth"))]
use blurmac::BluetoothGATTService as BluetoothGATTServiceMac;
#[cfg(feature = "bluetooth-test")]
use blurmock::fake_characteristic::FakeBluetoothGATTCharacteristic;
#[cfg(feature = "bluetooth-test")]
use blurmock::fake_descriptor::FakeBluetoothGATTDescriptor;
#[cfg(feature = "bluetooth-test")]
use blurmock::fake_device::FakeBluetoothDevice;
#[cfg(feature = "bluetooth-test")]
use blurmock::fake_discovery_session::FakeBluetoothDiscoverySession;
#[cfg(feature = "bluetooth-test")]
use blurmock::fake_service::FakeBluetoothGATTService;
#[cfg(all(target_os = "linux", feature = "native-bluetooth"))]
use blurz::bluetooth_device::BluetoothDevice as BluetoothDeviceBluez;
#[cfg(all(target_os = "linux", feature = "native-bluetooth"))]
use blurz::bluetooth_discovery_session::BluetoothDiscoverySession as BluetoothDiscoverySessionBluez;
#[cfg(all(target_os = "linux", feature = "native-bluetooth"))]
use blurz::bluetooth_gatt_characteristic::BluetoothGATTCharacteristic as BluetoothGATTCharacteristicBluez;
#[cfg(all(target_os = "linux", feature = "native-bluetooth"))]
use blurz::bluetooth_gatt_descriptor::BluetoothGATTDescriptor as BluetoothGATTDescriptorBluez;
#[cfg(all(target_os = "linux", feature = "native-bluetooth"))]
use blurz::bluetooth_gatt_service::BluetoothGATTService as BluetoothGATTServiceBluez;

pub use super::adapter::BluetoothAdapter;
#[cfg(not(any(
    all(target_os = "linux", feature = "native-bluetooth"),
    all(target_os = "android", feature = "native-bluetooth"),
    all(target_os = "macos", feature = "native-bluetooth")
)))]
use super::empty::BluetoothDevice as BluetoothDeviceEmpty;
#[cfg(not(any(
    all(target_os = "linux", feature = "native-bluetooth"),
    all(target_os = "android", feature = "native-bluetooth"),
    all(target_os = "macos", feature = "native-bluetooth")
)))]
use super::empty::BluetoothDiscoverySession as BluetoothDiscoverySessionEmpty;
#[cfg(not(any(
    all(target_os = "linux", feature = "native-bluetooth"),
    all(target_os = "android", feature = "native-bluetooth"),
    all(target_os = "macos", feature = "native-bluetooth")
)))]
use super::empty::BluetoothGATTCharacteristic as BluetoothGATTCharacteristicEmpty;
#[cfg(not(any(
    all(target_os = "linux", feature = "native-bluetooth"),
    all(target_os = "android", feature = "native-bluetooth"),
    all(target_os = "macos", feature = "native-bluetooth")
)))]
use super::empty::BluetoothGATTDescriptor as BluetoothGATTDescriptorEmpty;
#[cfg(not(any(
    all(target_os = "linux", feature = "native-bluetooth"),
    all(target_os = "android", feature = "native-bluetooth"),
    all(target_os = "macos", feature = "native-bluetooth")
)))]
use super::empty::BluetoothGATTService as BluetoothGATTServiceEmpty;
use super::macros::get_inner_and_call;
#[cfg(feature = "bluetooth-test")]
use super::macros::get_inner_and_call_test_func;

#[cfg(feature = "bluetooth-test")]
const NOT_SUPPORTED_ON_MOCK_ERROR: &str = "Error! The first parameter must be a mock structure!";

#[derive(Debug)]
pub enum BluetoothDiscoverySession {
    #[cfg(all(target_os = "linux", feature = "native-bluetooth"))]
    Bluez(Arc<BluetoothDiscoverySessionBluez>),
    #[cfg(all(target_os = "android", feature = "native-bluetooth"))]
    Android(Arc<BluetoothDiscoverySessionAndroid>),
    #[cfg(all(target_os = "macos", feature = "native-bluetooth"))]
    Mac(Arc<BluetoothDiscoverySessionMac>),
    #[cfg(not(any(
        all(target_os = "linux", feature = "native-bluetooth"),
        all(target_os = "android", feature = "native-bluetooth"),
        all(target_os = "macos", feature = "native-bluetooth")
    )))]
    Empty(Arc<BluetoothDiscoverySessionEmpty>),
    #[cfg(feature = "bluetooth-test")]
    Mock(Arc<FakeBluetoothDiscoverySession>),
}

#[derive(Clone, Debug)]
pub enum BluetoothDevice {
    #[cfg(all(target_os = "linux", feature = "native-bluetooth"))]
    Bluez(Arc<BluetoothDeviceBluez>),
    #[cfg(all(target_os = "android", feature = "native-bluetooth"))]
    Android(Arc<BluetoothDeviceAndroid>),
    #[cfg(all(target_os = "macos", feature = "native-bluetooth"))]
    Mac(Arc<BluetoothDeviceMac>),
    #[cfg(not(any(
        all(target_os = "linux", feature = "native-bluetooth"),
        all(target_os = "android", feature = "native-bluetooth"),
        all(target_os = "macos", feature = "native-bluetooth")
    )))]
    Empty(Arc<BluetoothDeviceEmpty>),
    #[cfg(feature = "bluetooth-test")]
    Mock(Arc<FakeBluetoothDevice>),
}

#[derive(Clone, Debug)]
pub enum BluetoothGATTService {
    #[cfg(all(target_os = "linux", feature = "native-bluetooth"))]
    Bluez(Arc<BluetoothGATTServiceBluez>),
    #[cfg(all(target_os = "android", feature = "native-bluetooth"))]
    Android(Arc<BluetoothGATTServiceAndroid>),
    #[cfg(all(target_os = "macos", feature = "native-bluetooth"))]
    Mac(Arc<BluetoothGATTServiceMac>),
    #[cfg(not(any(
        all(target_os = "linux", feature = "native-bluetooth"),
        all(target_os = "android", feature = "native-bluetooth"),
        all(target_os = "macos", feature = "native-bluetooth")
    )))]
    Empty(Arc<BluetoothGATTServiceEmpty>),
    #[cfg(feature = "bluetooth-test")]
    Mock(Arc<FakeBluetoothGATTService>),
}

#[derive(Clone, Debug)]
pub enum BluetoothGATTCharacteristic {
    #[cfg(all(target_os = "linux", feature = "native-bluetooth"))]
    Bluez(Arc<BluetoothGATTCharacteristicBluez>),
    #[cfg(all(target_os = "android", feature = "native-bluetooth"))]
    Android(Arc<BluetoothGATTCharacteristicAndroid>),
    #[cfg(all(target_os = "macos", feature = "native-bluetooth"))]
    Mac(Arc<BluetoothGATTCharacteristicMac>),
    #[cfg(not(any(
        all(target_os = "linux", feature = "native-bluetooth"),
        all(target_os = "android", feature = "native-bluetooth"),
        all(target_os = "macos", feature = "native-bluetooth")
    )))]
    Empty(Arc<BluetoothGATTCharacteristicEmpty>),
    #[cfg(feature = "bluetooth-test")]
    Mock(Arc<FakeBluetoothGATTCharacteristic>),
}

#[derive(Clone, Debug)]
pub enum BluetoothGATTDescriptor {
    #[cfg(all(target_os = "linux", feature = "native-bluetooth"))]
    Bluez(Arc<BluetoothGATTDescriptorBluez>),
    #[cfg(all(target_os = "android", feature = "native-bluetooth"))]
    Android(Arc<BluetoothGATTDescriptorAndroid>),
    #[cfg(all(target_os = "macos", feature = "native-bluetooth"))]
    Mac(Arc<BluetoothGATTDescriptorMac>),
    #[cfg(not(any(
        all(target_os = "linux", feature = "native-bluetooth"),
        all(target_os = "android", feature = "native-bluetooth"),
        all(target_os = "macos", feature = "native-bluetooth")
    )))]
    Empty(Arc<BluetoothGATTDescriptorEmpty>),
    #[cfg(feature = "bluetooth-test")]
    Mock(Arc<FakeBluetoothGATTDescriptor>),
}

impl BluetoothDiscoverySession {
    pub fn start_discovery(&self) -> Result<(), Box<dyn Error>> {
        get_inner_and_call!(self, BluetoothDiscoverySession, start_discovery)
    }

    pub fn stop_discovery(&self) -> Result<(), Box<dyn Error>> {
        get_inner_and_call!(self, BluetoothDiscoverySession, stop_discovery)
    }
}

impl BluetoothDevice {
    pub fn get_id(&self) -> String {
        get_inner_and_call!(self, BluetoothDevice, get_id)
    }

    #[cfg(feature = "bluetooth-test")]
    pub fn set_id(&self, id: String) {
        if let BluetoothDevice::Mock(fake_adapter) = self {
            fake_adapter.set_id(id)
        }
    }

    pub fn get_address(&self) -> Result<String, Box<dyn Error>> {
        get_inner_and_call!(self, BluetoothDevice, get_address)
    }

    #[cfg(feature = "bluetooth-test")]
    pub fn set_address(&self, address: String) -> Result<(), Box<dyn Error>> {
        get_inner_and_call_test_func!(self, BluetoothDevice, set_address, address)
    }

    pub fn get_name(&self) -> Result<String, Box<dyn Error>> {
        get_inner_and_call!(self, BluetoothDevice, get_name)
    }

    #[cfg(feature = "bluetooth-test")]
    pub fn set_name(&self, name: Option<String>) -> Result<(), Box<dyn Error>> {
        get_inner_and_call_test_func!(self, BluetoothDevice, set_name, name)
    }

    pub fn get_icon(&self) -> Result<String, Box<dyn Error>> {
        get_inner_and_call!(self, BluetoothDevice, get_icon)
    }

    #[cfg(feature = "bluetooth-test")]
    pub fn set_icon(&self, icon: String) -> Result<(), Box<dyn Error>> {
        get_inner_and_call_test_func!(self, BluetoothDevice, set_icon, icon)
    }

    pub fn get_class(&self) -> Result<u32, Box<dyn Error>> {
        get_inner_and_call!(self, BluetoothDevice, get_class)
    }

    #[cfg(feature = "bluetooth-test")]
    pub fn set_class(&self, class: u32) -> Result<(), Box<dyn Error>> {
        get_inner_and_call_test_func!(self, BluetoothDevice, set_class, class)
    }

    pub fn get_appearance(&self) -> Result<u16, Box<dyn Error>> {
        get_inner_and_call!(self, BluetoothDevice, get_appearance)
    }

    #[cfg(feature = "bluetooth-test")]
    pub fn set_appearance(&self, appearance: u16) -> Result<(), Box<dyn Error>> {
        get_inner_and_call_test_func!(self, BluetoothDevice, set_appearance, Some(appearance))
    }

    pub fn get_uuids(&self) -> Result<Vec<String>, Box<dyn Error>> {
        get_inner_and_call!(self, BluetoothDevice, get_uuids)
    }

    #[cfg(feature = "bluetooth-test")]
    pub fn set_uuids(&self, uuids: Vec<String>) -> Result<(), Box<dyn Error>> {
        get_inner_and_call_test_func!(self, BluetoothDevice, set_uuids, uuids)
    }

    pub fn is_paired(&self) -> Result<bool, Box<dyn Error>> {
        get_inner_and_call!(self, BluetoothDevice, is_paired)
    }

    #[cfg(feature = "bluetooth-test")]
    pub fn set_paired(&self, paired: bool) -> Result<(), Box<dyn Error>> {
        get_inner_and_call_test_func!(self, BluetoothDevice, set_paired, paired)
    }

    pub fn is_connected(&self) -> Result<bool, Box<dyn Error>> {
        get_inner_and_call!(self, BluetoothDevice, is_connected)
    }

    #[cfg(feature = "bluetooth-test")]
    pub fn set_connected(&self, connected: bool) -> Result<(), Box<dyn Error>> {
        get_inner_and_call_test_func!(self, BluetoothDevice, set_connected, connected)
    }

    #[cfg(feature = "bluetooth-test")]
    pub fn is_connectable(&self) -> Result<bool, Box<dyn Error>> {
        get_inner_and_call_test_func!(self, BluetoothDevice, is_connectable)
    }

    #[cfg(feature = "bluetooth-test")]
    pub fn set_connectable(&self, connectable: bool) -> Result<(), Box<dyn Error>> {
        get_inner_and_call_test_func!(self, BluetoothDevice, set_connectable, connectable)
    }

    pub fn is_trusted(&self) -> Result<bool, Box<dyn Error>> {
        get_inner_and_call!(self, BluetoothDevice, is_trusted)
    }

    #[cfg(feature = "bluetooth-test")]
    pub fn set_trusted(&self, trusted: bool) -> Result<(), Box<dyn Error>> {
        get_inner_and_call_test_func!(self, BluetoothDevice, set_trusted, trusted)
    }

    pub fn is_blocked(&self) -> Result<bool, Box<dyn Error>> {
        get_inner_and_call!(self, BluetoothDevice, is_blocked)
    }

    #[cfg(feature = "bluetooth-test")]
    pub fn set_blocked(&self, blocked: bool) -> Result<(), Box<dyn Error>> {
        get_inner_and_call_test_func!(self, BluetoothDevice, set_blocked, blocked)
    }

    pub fn get_alias(&self) -> Result<String, Box<dyn Error>> {
        get_inner_and_call!(self, BluetoothDevice, get_alias)
    }

    #[cfg(feature = "bluetooth-test")]
    pub fn set_alias(&self, alias: String) -> Result<(), Box<dyn Error>> {
        get_inner_and_call_test_func!(self, BluetoothDevice, set_alias, alias)
    }

    pub fn is_legacy_pairing(&self) -> Result<bool, Box<dyn Error>> {
        get_inner_and_call!(self, BluetoothDevice, is_legacy_pairing)
    }

    #[cfg(feature = "bluetooth-test")]
    pub fn set_legacy_pairing(&self, legacy_pairing: bool) -> Result<(), Box<dyn Error>> {
        get_inner_and_call_test_func!(self, BluetoothDevice, set_legacy_pairing, legacy_pairing)
    }

    pub fn get_vendor_id_source(&self) -> Result<String, Box<dyn Error>> {
        get_inner_and_call!(self, BluetoothDevice, get_vendor_id_source)
    }

    pub fn get_vendor_id(&self) -> Result<u32, Box<dyn Error>> {
        get_inner_and_call!(self, BluetoothDevice, get_vendor_id)
    }

    pub fn get_product_id(&self) -> Result<u32, Box<dyn Error>> {
        get_inner_and_call!(self, BluetoothDevice, get_product_id)
    }

    pub fn get_device_id(&self) -> Result<u32, Box<dyn Error>> {
        get_inner_and_call!(self, BluetoothDevice, get_device_id)
    }

    pub fn get_modalias(&self) -> Result<(String, u32, u32, u32), Box<dyn Error>> {
        get_inner_and_call!(self, BluetoothDevice, get_modalias)
    }

    #[cfg(feature = "bluetooth-test")]
    pub fn set_modalias(&self, modalias: String) -> Result<(), Box<dyn Error>> {
        get_inner_and_call_test_func!(self, BluetoothDevice, set_modalias, modalias)
    }

    pub fn get_rssi(&self) -> Result<i16, Box<dyn Error>> {
        get_inner_and_call!(self, BluetoothDevice, get_rssi)
    }

    #[cfg(feature = "bluetooth-test")]
    pub fn set_rssi(&self, rssi: i16) -> Result<(), Box<dyn Error>> {
        get_inner_and_call_test_func!(self, BluetoothDevice, set_rssi, Some(rssi))
    }

    pub fn get_tx_power(&self) -> Result<i16, Box<dyn Error>> {
        get_inner_and_call!(self, BluetoothDevice, get_tx_power)
    }

    #[cfg(feature = "bluetooth-test")]
    pub fn set_tx_power(&self, tx_power: i16) -> Result<(), Box<dyn Error>> {
        get_inner_and_call_test_func!(self, BluetoothDevice, set_tx_power, Some(tx_power))
    }

    pub fn get_manufacturer_data(&self) -> Result<HashMap<u16, Vec<u8>>, Box<dyn Error>> {
        get_inner_and_call!(self, BluetoothDevice, get_manufacturer_data)
    }

    #[cfg(feature = "bluetooth-test")]
    pub fn set_manufacturer_data(
        &self,
        manufacturer_data: HashMap<u16, Vec<u8>>,
    ) -> Result<(), Box<dyn Error>> {
        get_inner_and_call_test_func!(
            self,
            BluetoothDevice,
            set_manufacturer_data,
            Some(manufacturer_data)
        )
    }

    pub fn get_service_data(&self) -> Result<HashMap<String, Vec<u8>>, Box<dyn Error>> {
        get_inner_and_call!(self, BluetoothDevice, get_service_data)
    }

    #[cfg(feature = "bluetooth-test")]
    pub fn set_service_data(
        &self,
        service_data: HashMap<String, Vec<u8>>,
    ) -> Result<(), Box<dyn Error>> {
        get_inner_and_call_test_func!(self, BluetoothDevice, set_service_data, Some(service_data))
    }

    pub fn get_gatt_services(&self) -> Result<Vec<BluetoothGATTService>, Box<dyn Error>> {
        let services = get_inner_and_call!(self, BluetoothDevice, get_gatt_services)?;
        Ok(services
            .into_iter()
            .map(|service| BluetoothGATTService::create_service(self.clone(), service))
            .collect())
    }

    pub fn connect(&self) -> Result<(), Box<dyn Error>> {
        get_inner_and_call!(self, BluetoothDevice, connect)
    }

    pub fn disconnect(&self) -> Result<(), Box<dyn Error>> {
        get_inner_and_call!(self, BluetoothDevice, disconnect)
    }

    pub fn connect_profile(&self, uuid: String) -> Result<(), Box<dyn Error>> {
        get_inner_and_call!(self, BluetoothDevice, connect_profile, uuid)
    }

    pub fn disconnect_profile(&self, uuid: String) -> Result<(), Box<dyn Error>> {
        get_inner_and_call!(self, BluetoothDevice, disconnect_profile, uuid)
    }

    pub fn pair(&self) -> Result<(), Box<dyn Error>> {
        get_inner_and_call!(self, BluetoothDevice, pair)
    }

    pub fn cancel_pairing(&self) -> Result<(), Box<dyn Error>> {
        get_inner_and_call!(self, BluetoothDevice, cancel_pairing)
    }
}

impl BluetoothGATTService {
    fn create_service(device: BluetoothDevice, service: String) -> BluetoothGATTService {
        match device {
            #[cfg(all(target_os = "linux", feature = "native-bluetooth"))]
            BluetoothDevice::Bluez(_bluez_device) => {
                BluetoothGATTService::Bluez(Arc::new(BluetoothGATTServiceBluez::new(service)))
            },
            #[cfg(all(target_os = "android", feature = "native-bluetooth"))]
            BluetoothDevice::Android(android_device) => BluetoothGATTService::Android(Arc::new(
                BluetoothGATTServiceAndroid::new(android_device, service),
            )),
            #[cfg(all(target_os = "macos", feature = "native-bluetooth"))]
            BluetoothDevice::Mac(mac_device) => BluetoothGATTService::Mac(Arc::new(
                BluetoothGATTServiceMac::new(mac_device, service),
            )),
            #[cfg(not(any(
                all(target_os = "linux", feature = "native-bluetooth"),
                all(target_os = "android", feature = "native-bluetooth"),
                all(target_os = "macos", feature = "native-bluetooth")
            )))]
            BluetoothDevice::Empty(_device) => {
                BluetoothGATTService::Empty(Arc::new(BluetoothGATTServiceEmpty::new(service)))
            },
            #[cfg(feature = "bluetooth-test")]
            BluetoothDevice::Mock(fake_device) => BluetoothGATTService::Mock(
                FakeBluetoothGATTService::new_empty(fake_device, service),
            ),
        }
    }

    #[cfg(feature = "bluetooth-test")]
    pub fn create_mock_service(
        device: BluetoothDevice,
        service: String,
    ) -> Result<BluetoothGATTService, Box<dyn Error>> {
        match device {
            BluetoothDevice::Mock(fake_device) => Ok(BluetoothGATTService::Mock(
                FakeBluetoothGATTService::new_empty(fake_device, service),
            )),
            _ => Err(Box::from(
                "Error! The first parameter must be a mock structure!",
            )),
        }
    }

    pub fn get_id(&self) -> String {
        get_inner_and_call!(self, BluetoothGATTService, get_id)
    }

    #[cfg(feature = "bluetooth-test")]
    pub fn set_id(&self, id: String) {
        if let BluetoothGATTService::Mock(fake_service) = self {
            fake_service.set_id(id)
        }
    }

    pub fn get_uuid(&self) -> Result<String, Box<dyn Error>> {
        get_inner_and_call!(self, BluetoothGATTService, get_uuid)
    }

    #[cfg(feature = "bluetooth-test")]
    pub fn set_uuid(&self, uuid: String) -> Result<(), Box<dyn Error>> {
        get_inner_and_call_test_func!(self, BluetoothGATTService, set_uuid, uuid)
    }

    pub fn is_primary(&self) -> Result<bool, Box<dyn Error>> {
        get_inner_and_call!(self, BluetoothGATTService, is_primary)
    }

    #[cfg(feature = "bluetooth-test")]
    pub fn set_primary(&self, primary: bool) -> Result<(), Box<dyn Error>> {
        get_inner_and_call_test_func!(self, BluetoothGATTService, set_is_primary, primary)
    }

    pub fn get_includes(
        &self,
        device: BluetoothDevice,
    ) -> Result<Vec<BluetoothGATTService>, Box<dyn Error>> {
        let services = get_inner_and_call!(self, BluetoothGATTService, get_includes)?;
        Ok(services
            .into_iter()
            .map(|service| BluetoothGATTService::create_service(device.clone(), service))
            .collect())
    }

    pub fn get_gatt_characteristics(
        &self,
    ) -> Result<Vec<BluetoothGATTCharacteristic>, Box<dyn Error>> {
        let characteristics =
            get_inner_and_call!(self, BluetoothGATTService, get_gatt_characteristics)?;
        Ok(characteristics
            .into_iter()
            .map(|characteristic| {
                BluetoothGATTCharacteristic::create_characteristic(self.clone(), characteristic)
            })
            .collect())
    }
}

impl BluetoothGATTCharacteristic {
    fn create_characteristic(
        service: BluetoothGATTService,
        characteristic: String,
    ) -> BluetoothGATTCharacteristic {
        match service {
            #[cfg(all(target_os = "linux", feature = "native-bluetooth"))]
            BluetoothGATTService::Bluez(_bluez_service) => BluetoothGATTCharacteristic::Bluez(
                Arc::new(BluetoothGATTCharacteristicBluez::new(characteristic)),
            ),
            #[cfg(all(target_os = "android", feature = "native-bluetooth"))]
            BluetoothGATTService::Android(android_service) => {
                BluetoothGATTCharacteristic::Android(Arc::new(
                    BluetoothGATTCharacteristicAndroid::new(android_service, characteristic),
                ))
            },
            #[cfg(all(target_os = "macos", feature = "native-bluetooth"))]
            BluetoothGATTService::Mac(mac_service) => BluetoothGATTCharacteristic::Mac(Arc::new(
                BluetoothGATTCharacteristicMac::new(mac_service, characteristic),
            )),
            #[cfg(not(any(
                all(target_os = "linux", feature = "native-bluetooth"),
                all(target_os = "android", feature = "native-bluetooth"),
                all(target_os = "macos", feature = "native-bluetooth")
            )))]
            BluetoothGATTService::Empty(_service) => BluetoothGATTCharacteristic::Empty(Arc::new(
                BluetoothGATTCharacteristicEmpty::new(characteristic),
            )),
            #[cfg(feature = "bluetooth-test")]
            BluetoothGATTService::Mock(fake_service) => BluetoothGATTCharacteristic::Mock(
                FakeBluetoothGATTCharacteristic::new_empty(fake_service, characteristic),
            ),
        }
    }

    #[cfg(feature = "bluetooth-test")]
    pub fn create_mock_characteristic(
        service: BluetoothGATTService,
        characteristic: String,
    ) -> Result<BluetoothGATTCharacteristic, Box<dyn Error>> {
        match service {
            BluetoothGATTService::Mock(fake_service) => Ok(BluetoothGATTCharacteristic::Mock(
                FakeBluetoothGATTCharacteristic::new_empty(fake_service, characteristic),
            )),
            _ => Err(Box::from(
                "Error! The first parameter must be a mock structure!",
            )),
        }
    }

    pub fn get_id(&self) -> String {
        get_inner_and_call!(self, BluetoothGATTCharacteristic, get_id)
    }

    #[cfg(feature = "bluetooth-test")]
    pub fn set_id(&self, id: String) {
        if let BluetoothGATTCharacteristic::Mock(fake_characteristic) = self {
            fake_characteristic.set_id(id)
        }
    }

    pub fn get_uuid(&self) -> Result<String, Box<dyn Error>> {
        get_inner_and_call!(self, BluetoothGATTCharacteristic, get_uuid)
    }

    #[cfg(feature = "bluetooth-test")]
    pub fn set_uuid(&self, uuid: String) -> Result<(), Box<dyn Error>> {
        get_inner_and_call_test_func!(self, BluetoothGATTCharacteristic, set_uuid, uuid)
    }

    pub fn get_value(&self) -> Result<Vec<u8>, Box<dyn Error>> {
        get_inner_and_call!(self, BluetoothGATTCharacteristic, get_value)
    }

    #[cfg(feature = "bluetooth-test")]
    pub fn set_value(&self, value: Vec<u8>) -> Result<(), Box<dyn Error>> {
        get_inner_and_call_test_func!(self, BluetoothGATTCharacteristic, set_value, Some(value))
    }

    pub fn is_notifying(&self) -> Result<bool, Box<dyn Error>> {
        get_inner_and_call!(self, BluetoothGATTCharacteristic, is_notifying)
    }

    #[cfg(feature = "bluetooth-test")]
    pub fn set_notifying(&self, notifying: bool) -> Result<(), Box<dyn Error>> {
        get_inner_and_call_test_func!(self, BluetoothGATTCharacteristic, set_notifying, notifying)
    }

    pub fn get_flags(&self) -> Result<Vec<String>, Box<dyn Error>> {
        get_inner_and_call!(self, BluetoothGATTCharacteristic, get_flags)
    }

    #[cfg(feature = "bluetooth-test")]
    pub fn set_flags(&self, flags: Vec<String>) -> Result<(), Box<dyn Error>> {
        get_inner_and_call_test_func!(self, BluetoothGATTCharacteristic, set_flags, flags)
    }

    pub fn get_gatt_descriptors(&self) -> Result<Vec<BluetoothGATTDescriptor>, Box<dyn Error>> {
        let descriptors =
            get_inner_and_call!(self, BluetoothGATTCharacteristic, get_gatt_descriptors)?;
        Ok(descriptors
            .into_iter()
            .map(|descriptor| BluetoothGATTDescriptor::create_descriptor(self.clone(), descriptor))
            .collect())
    }

    pub fn read_value(&self) -> Result<Vec<u8>, Box<dyn Error>> {
        get_inner_and_call!(@with_bluez_offset, self, BluetoothGATTCharacteristic, read_value)
    }

    pub fn write_value(&self, values: Vec<u8>) -> Result<(), Box<dyn Error>> {
        get_inner_and_call!(@with_bluez_offset, self, BluetoothGATTCharacteristic, write_value, values)
    }

    pub fn start_notify(&self) -> Result<(), Box<dyn Error>> {
        get_inner_and_call!(self, BluetoothGATTCharacteristic, start_notify)
    }

    pub fn stop_notify(&self) -> Result<(), Box<dyn Error>> {
        get_inner_and_call!(self, BluetoothGATTCharacteristic, stop_notify)
    }
}

impl BluetoothGATTDescriptor {
    fn create_descriptor(
        characteristic: BluetoothGATTCharacteristic,
        descriptor: String,
    ) -> BluetoothGATTDescriptor {
        match characteristic {
            #[cfg(all(target_os = "linux", feature = "native-bluetooth"))]
            BluetoothGATTCharacteristic::Bluez(_bluez_characteristic) => {
                BluetoothGATTDescriptor::Bluez(Arc::new(BluetoothGATTDescriptorBluez::new(
                    descriptor,
                )))
            },
            #[cfg(all(target_os = "android", feature = "native-bluetooth"))]
            BluetoothGATTCharacteristic::Android(android_characteristic) => {
                BluetoothGATTDescriptor::Android(Arc::new(BluetoothGATTDescriptorAndroid::new(
                    android_characteristic,
                    descriptor,
                )))
            },
            #[cfg(all(target_os = "macos", feature = "native-bluetooth"))]
            BluetoothGATTCharacteristic::Mac(_mac_characteristic) => {
                BluetoothGATTDescriptor::Mac(Arc::new(BluetoothGATTDescriptorMac::new(descriptor)))
            },
            #[cfg(not(any(
                all(target_os = "linux", feature = "native-bluetooth"),
                all(target_os = "android", feature = "native-bluetooth"),
                all(target_os = "macos", feature = "native-bluetooth")
            )))]
            BluetoothGATTCharacteristic::Empty(_characteristic) => BluetoothGATTDescriptor::Empty(
                Arc::new(BluetoothGATTDescriptorEmpty::new(descriptor)),
            ),
            #[cfg(feature = "bluetooth-test")]
            BluetoothGATTCharacteristic::Mock(fake_characteristic) => {
                BluetoothGATTDescriptor::Mock(FakeBluetoothGATTDescriptor::new_empty(
                    fake_characteristic,
                    descriptor,
                ))
            },
        }
    }

    #[cfg(feature = "bluetooth-test")]
    pub fn create_mock_descriptor(
        characteristic: BluetoothGATTCharacteristic,
        descriptor: String,
    ) -> Result<BluetoothGATTDescriptor, Box<dyn Error>> {
        match characteristic {
            BluetoothGATTCharacteristic::Mock(fake_characteristic) => {
                Ok(BluetoothGATTDescriptor::Mock(
                    FakeBluetoothGATTDescriptor::new_empty(fake_characteristic, descriptor),
                ))
            },
            _ => Err(Box::from(NOT_SUPPORTED_ON_MOCK_ERROR)),
        }
    }

    pub fn get_id(&self) -> String {
        get_inner_and_call!(self, BluetoothGATTDescriptor, get_id)
    }

    #[cfg(feature = "bluetooth-test")]
    pub fn set_id(&self, id: String) {
        if let BluetoothGATTDescriptor::Mock(fake_descriptor) = self {
            fake_descriptor.set_id(id)
        }
    }

    pub fn get_uuid(&self) -> Result<String, Box<dyn Error>> {
        get_inner_and_call!(self, BluetoothGATTDescriptor, get_uuid)
    }

    #[cfg(feature = "bluetooth-test")]
    pub fn set_uuid(&self, uuid: String) -> Result<(), Box<dyn Error>> {
        get_inner_and_call_test_func!(self, BluetoothGATTDescriptor, set_uuid, uuid)
    }

    pub fn get_value(&self) -> Result<Vec<u8>, Box<dyn Error>> {
        get_inner_and_call!(self, BluetoothGATTDescriptor, get_value)
    }

    #[cfg(feature = "bluetooth-test")]
    pub fn set_value(&self, value: Vec<u8>) -> Result<(), Box<dyn Error>> {
        get_inner_and_call_test_func!(self, BluetoothGATTDescriptor, set_value, Some(value))
    }

    pub fn get_flags(&self) -> Result<Vec<String>, Box<dyn Error>> {
        get_inner_and_call!(self, BluetoothGATTDescriptor, get_flags)
    }

    #[cfg(feature = "bluetooth-test")]
    pub fn set_flags(&self, flags: Vec<String>) -> Result<(), Box<dyn Error>> {
        get_inner_and_call_test_func!(self, BluetoothGATTDescriptor, set_flags, flags)
    }

    pub fn read_value(&self) -> Result<Vec<u8>, Box<dyn Error>> {
        get_inner_and_call!(@with_bluez_offset, self, BluetoothGATTDescriptor, read_value)
    }

    pub fn write_value(&self, values: Vec<u8>) -> Result<(), Box<dyn Error>> {
        get_inner_and_call!(@with_bluez_offset, self, BluetoothGATTDescriptor, write_value, values)
    }
}
