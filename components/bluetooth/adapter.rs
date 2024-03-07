/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use std::error::Error;
use std::sync::Arc;

#[cfg(all(target_os = "android", feature = "native-bluetooth"))]
use blurdroid::bluetooth_adapter::Adapter as BluetoothAdapterAndroid;
#[cfg(all(target_os = "android", feature = "native-bluetooth"))]
use blurdroid::bluetooth_device::Device as BluetoothDeviceAndroid;
#[cfg(all(target_os = "android", feature = "native-bluetooth"))]
use blurdroid::bluetooth_discovery_session::DiscoverySession as BluetoothDiscoverySessionAndroid;
#[cfg(all(target_os = "macos", feature = "native-bluetooth"))]
use blurmac::BluetoothAdapter as BluetoothAdapterMac;
#[cfg(all(target_os = "macos", feature = "native-bluetooth"))]
use blurmac::BluetoothDevice as BluetoothDeviceMac;
#[cfg(all(target_os = "macos", feature = "native-bluetooth"))]
use blurmac::BluetoothDiscoverySession as BluetoothDiscoverySessionMac;
#[cfg(feature = "bluetooth-test")]
use blurmock::fake_adapter::FakeBluetoothAdapter;
#[cfg(feature = "bluetooth-test")]
use blurmock::fake_device::FakeBluetoothDevice;
#[cfg(feature = "bluetooth-test")]
use blurmock::fake_discovery_session::FakeBluetoothDiscoverySession;
#[cfg(all(target_os = "linux", feature = "native-bluetooth"))]
use blurz::bluetooth_adapter::BluetoothAdapter as BluetoothAdapterBluez;
#[cfg(all(target_os = "linux", feature = "native-bluetooth"))]
use blurz::bluetooth_device::BluetoothDevice as BluetoothDeviceBluez;
#[cfg(all(target_os = "linux", feature = "native-bluetooth"))]
use blurz::bluetooth_discovery_session::BluetoothDiscoverySession as BluetoothDiscoverySessionBluez;

use super::bluetooth::{BluetoothDevice, BluetoothDiscoverySession};
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
use super::empty::EmptyAdapter as BluetoothAdapterEmpty;
use super::macros::get_inner_and_call;
#[cfg(feature = "bluetooth-test")]
use super::macros::get_inner_and_call_test_func;

#[derive(Clone, Debug)]
pub enum BluetoothAdapter {
    #[cfg(all(target_os = "linux", feature = "native-bluetooth"))]
    Bluez(Arc<BluetoothAdapterBluez>),
    #[cfg(all(target_os = "android", feature = "native-bluetooth"))]
    Android(Arc<BluetoothAdapterAndroid>),
    #[cfg(all(target_os = "macos", feature = "native-bluetooth"))]
    Mac(Arc<BluetoothAdapterMac>),
    #[cfg(not(any(
        all(target_os = "linux", feature = "native-bluetooth"),
        all(target_os = "android", feature = "native-bluetooth"),
        all(target_os = "macos", feature = "native-bluetooth")
    )))]
    Empty(Arc<BluetoothAdapterEmpty>),
    #[cfg(feature = "bluetooth-test")]
    Mock(Arc<FakeBluetoothAdapter>),
}

impl BluetoothAdapter {
    #[cfg(all(target_os = "linux", feature = "native-bluetooth"))]
    pub fn new() -> Result<BluetoothAdapter, Box<dyn Error>> {
        let bluez_adapter = BluetoothAdapterBluez::init()?;
        Ok(Self::Bluez(Arc::new(bluez_adapter)))
    }

    #[cfg(all(target_os = "android", feature = "native-bluetooth"))]
    pub fn new() -> Result<BluetoothAdapter, Box<dyn Error>> {
        let blurdroid_adapter = BluetoothAdapterAndroid::get_adapter()?;
        Ok(Self::Android(Arc::new(blurdroid_adapter)))
    }

    #[cfg(all(target_os = "macos", feature = "native-bluetooth"))]
    pub fn new() -> Result<BluetoothAdapter, Box<dyn Error>> {
        let mac_adapter = BluetoothAdapterMac::init()?;
        Ok(Self::Mac(Arc::new(mac_adapter)))
    }

    #[cfg(not(any(
        all(target_os = "linux", feature = "native-bluetooth"),
        all(target_os = "android", feature = "native-bluetooth"),
        all(target_os = "macos", feature = "native-bluetooth")
    )))]
    pub fn new() -> Result<BluetoothAdapter, Box<dyn Error>> {
        let adapter = BluetoothAdapterEmpty::init()?;
        Ok(Self::Empty(Arc::new(adapter)))
    }

    #[cfg(feature = "bluetooth-test")]
    pub fn new_mock() -> Result<BluetoothAdapter, Box<dyn Error>> {
        Ok(Self::Mock(FakeBluetoothAdapter::new_empty()))
    }

    pub fn get_id(&self) -> String {
        get_inner_and_call!(self, BluetoothAdapter, get_id)
    }

    pub fn get_devices(&self) -> Result<Vec<BluetoothDevice>, Box<dyn Error>> {
        match self {
            #[cfg(all(target_os = "linux", feature = "native-bluetooth"))]
            BluetoothAdapter::Bluez(inner) => {
                let device_list = inner.get_device_list()?;
                Ok(device_list
                    .into_iter()
                    .map(|device| BluetoothDevice::Bluez(BluetoothDeviceBluez::new(device).into()))
                    .collect())
            },
            #[cfg(all(target_os = "android", feature = "native-bluetooth"))]
            BluetoothAdapter::Android(inner) => {
                let device_list = inner.get_device_list()?;
                Ok(device_list
                    .into_iter()
                    .map(|device| {
                        BluetoothDevice::Android(BluetoothDeviceAndroid::new_empty(
                            self.0.clone(),
                            device,
                        ))
                    })
                    .collect())
            },
            #[cfg(all(target_os = "macos", feature = "native-bluetooth"))]
            BluetoothAdapter::Mac(inner) => {
                let device_list = inner.get_device_list()?;
                Ok(device_list
                    .into_iter()
                    .map(|device| {
                        BluetoothDevice::Mac(Arc::new(BluetoothDeviceMac::new(
                            inner.clone(),
                            device,
                        )))
                    })
                    .collect())
            },
            #[cfg(not(any(
                all(target_os = "linux", feature = "native-bluetooth"),
                all(target_os = "android", feature = "native-bluetooth"),
                all(target_os = "macos", feature = "native-bluetooth")
            )))]
            BluetoothAdapter::Empty(inner) => {
                let device_list = inner.get_device_list()?;
                Ok(device_list
                    .into_iter()
                    .map(|device| {
                        BluetoothDevice::Empty(Arc::new(BluetoothDeviceEmpty::new(device)))
                    })
                    .collect())
            },
            #[cfg(feature = "bluetooth-test")]
            BluetoothAdapter::Mock(inner) => {
                let device_list = inner.get_device_list()?;
                Ok(device_list
                    .into_iter()
                    .map(|device| {
                        BluetoothDevice::Mock(FakeBluetoothDevice::new_empty(inner.clone(), device))
                    })
                    .collect())
            },
        }
    }

    pub fn get_device(&self, address: String) -> Result<Option<BluetoothDevice>, Box<dyn Error>> {
        let devices = self.get_devices()?;
        for device in devices {
            if device.get_address()? == address {
                return Ok(Some(device));
            }
        }
        Ok(None)
    }

    pub fn create_mock_device(&self, _device: String) -> Result<BluetoothDevice, Box<dyn Error>> {
        match self {
            #[cfg(feature = "bluetooth-test")]
            BluetoothAdapter::Mock(inner) => Ok(BluetoothDevice::Mock(
                FakeBluetoothDevice::new_empty(inner.clone(), _device),
            )),
            _ => Err(Box::from(
                "Error! Test functions are not supported on real devices!",
            )),
        }
    }

    pub fn create_discovery_session(&self) -> Result<BluetoothDiscoverySession, Box<dyn Error>> {
        let discovery_session = match self {
            #[cfg(all(target_os = "linux", feature = "native-bluetooth"))]
            #[allow(clippy::arc_with_non_send_sync)] // Problem with underlying library
            BluetoothAdapter::Bluez(inner) => BluetoothDiscoverySession::Bluez(Arc::new(
                BluetoothDiscoverySessionBluez::create_session(inner.get_id())?,
            )),
            #[cfg(all(target_os = "android", feature = "native-bluetooth"))]
            BluetoothAdapter::Android(inner) => BluetoothDiscoverySession::Android(Arc::new(
                BluetoothDiscoverySessionAndroid::create_session(inner.clone())?,
            )),
            #[cfg(all(target_os = "macos", feature = "native-bluetooth"))]
            BluetoothAdapter::Mac(_) => {
                BluetoothDiscoverySession::Mac(Arc::new(BluetoothDiscoverySessionMac {}))
            },
            #[cfg(not(any(
                all(target_os = "linux", feature = "native-bluetooth"),
                all(target_os = "android", feature = "native-bluetooth"),
                all(target_os = "macos", feature = "native-bluetooth")
            )))]
            BluetoothAdapter::Empty(_) => {
                BluetoothDiscoverySession::Empty(Arc::new(BluetoothDiscoverySessionEmpty {}))
            },
            #[cfg(feature = "bluetooth-test")]
            BluetoothAdapter::Mock(inner) => BluetoothDiscoverySession::Mock(Arc::new(
                FakeBluetoothDiscoverySession::create_session(inner.clone())?,
            )),
        };
        Ok(discovery_session)
    }

    pub fn get_address(&self) -> Result<String, Box<dyn Error>> {
        get_inner_and_call!(self, BluetoothAdapter, get_address)
    }

    pub fn get_name(&self) -> Result<String, Box<dyn Error>> {
        get_inner_and_call!(self, BluetoothAdapter, get_name)
    }

    pub fn get_alias(&self) -> Result<String, Box<dyn Error>> {
        get_inner_and_call!(self, BluetoothAdapter, get_alias)
    }

    pub fn get_class(&self) -> Result<u32, Box<dyn Error>> {
        get_inner_and_call!(self, BluetoothAdapter, get_class)
    }

    pub fn is_powered(&self) -> Result<bool, Box<dyn Error>> {
        get_inner_and_call!(self, BluetoothAdapter, is_powered)
    }

    pub fn is_discoverable(&self) -> Result<bool, Box<dyn Error>> {
        get_inner_and_call!(self, BluetoothAdapter, is_discoverable)
    }

    pub fn is_pairable(&self) -> Result<bool, Box<dyn Error>> {
        get_inner_and_call!(self, BluetoothAdapter, is_pairable)
    }

    pub fn get_pairable_timeout(&self) -> Result<u32, Box<dyn Error>> {
        get_inner_and_call!(self, BluetoothAdapter, get_pairable_timeout)
    }

    pub fn get_discoverable_timeout(&self) -> Result<u32, Box<dyn Error>> {
        get_inner_and_call!(self, BluetoothAdapter, get_discoverable_timeout)
    }

    pub fn is_discovering(&self) -> Result<bool, Box<dyn Error>> {
        get_inner_and_call!(self, BluetoothAdapter, is_discovering)
    }

    pub fn get_uuids(&self) -> Result<Vec<String>, Box<dyn Error>> {
        get_inner_and_call!(self, BluetoothAdapter, get_uuids)
    }

    pub fn get_vendor_id_source(&self) -> Result<String, Box<dyn Error>> {
        get_inner_and_call!(self, BluetoothAdapter, get_vendor_id_source)
    }

    pub fn get_vendor_id(&self) -> Result<u32, Box<dyn Error>> {
        get_inner_and_call!(self, BluetoothAdapter, get_vendor_id)
    }

    pub fn get_product_id(&self) -> Result<u32, Box<dyn Error>> {
        get_inner_and_call!(self, BluetoothAdapter, get_product_id)
    }

    pub fn get_device_id(&self) -> Result<u32, Box<dyn Error>> {
        get_inner_and_call!(self, BluetoothAdapter, get_device_id)
    }

    pub fn get_modalias(&self) -> Result<(String, u32, u32, u32), Box<dyn Error>> {
        get_inner_and_call!(self, BluetoothAdapter, get_modalias)
    }

    #[cfg(feature = "bluetooth-test")]
    pub fn set_id(&self, id: String) -> Result<(), Box<dyn Error>> {
        match self {
            #[cfg(feature = "bluetooth-test")]
            BluetoothAdapter::Mock(inner) => {
                inner.set_id(id);
                Ok(())
            },
            _ => Err(Box::from(
                "Error! Test functions are not supported on real devices!",
            )),
        }
    }

    #[cfg(feature = "bluetooth-test")]
    pub fn set_address(&self, address: String) -> Result<(), Box<dyn Error>> {
        get_inner_and_call_test_func!(self, BluetoothAdapter, set_address, address)
    }

    #[cfg(feature = "bluetooth-test")]
    pub fn set_name(&self, name: String) -> Result<(), Box<dyn Error>> {
        get_inner_and_call_test_func!(self, BluetoothAdapter, set_name, name)
    }

    #[cfg(feature = "bluetooth-test")]
    pub fn set_alias(&self, alias: String) -> Result<(), Box<dyn Error>> {
        get_inner_and_call_test_func!(self, BluetoothAdapter, set_alias, alias)
    }

    #[cfg(feature = "bluetooth-test")]
    pub fn set_class(&self, class: u32) -> Result<(), Box<dyn Error>> {
        get_inner_and_call_test_func!(self, BluetoothAdapter, set_class, class)
    }

    #[cfg(feature = "bluetooth-test")]
    pub fn set_powered(&self, powered: bool) -> Result<(), Box<dyn Error>> {
        get_inner_and_call_test_func!(self, BluetoothAdapter, set_powered, powered)
    }

    #[cfg(feature = "bluetooth-test")]
    pub fn is_present(&self) -> Result<bool, Box<dyn Error>> {
        get_inner_and_call_test_func!(self, BluetoothAdapter, is_present)
    }

    #[cfg(feature = "bluetooth-test")]
    pub fn set_present(&self, present: bool) -> Result<(), Box<dyn Error>> {
        get_inner_and_call_test_func!(self, BluetoothAdapter, set_present, present)
    }

    #[cfg(feature = "bluetooth-test")]
    pub fn set_discoverable(&self, discoverable: bool) -> Result<(), Box<dyn Error>> {
        get_inner_and_call_test_func!(self, BluetoothAdapter, set_discoverable, discoverable)
    }

    #[cfg(feature = "bluetooth-test")]
    pub fn set_pairable(&self, pairable: bool) -> Result<(), Box<dyn Error>> {
        get_inner_and_call_test_func!(self, BluetoothAdapter, set_pairable, pairable)
    }

    #[cfg(feature = "bluetooth-test")]
    pub fn set_pairable_timeout(&self, timeout: u32) -> Result<(), Box<dyn Error>> {
        get_inner_and_call_test_func!(self, BluetoothAdapter, set_pairable_timeout, timeout)
    }

    #[cfg(feature = "bluetooth-test")]
    pub fn set_can_start_discovery(&self, can_start_discovery: bool) -> Result<(), Box<dyn Error>> {
        get_inner_and_call_test_func!(
            self,
            BluetoothAdapter,
            set_can_start_discovery,
            can_start_discovery
        )
    }

    #[cfg(feature = "bluetooth-test")]
    pub fn set_discoverable_timeout(&self, timeout: u32) -> Result<(), Box<dyn Error>> {
        get_inner_and_call_test_func!(self, BluetoothAdapter, set_discoverable_timeout, timeout)
    }

    #[cfg(feature = "bluetooth-test")]
    pub fn set_discovering(&self, discovering: bool) -> Result<(), Box<dyn Error>> {
        get_inner_and_call_test_func!(self, BluetoothAdapter, set_discovering, discovering)
    }

    #[cfg(feature = "bluetooth-test")]
    pub fn set_can_stop_discovery(&self, can_stop_discovery: bool) -> Result<(), Box<dyn Error>> {
        get_inner_and_call_test_func!(
            self,
            BluetoothAdapter,
            set_can_stop_discovery,
            can_stop_discovery
        )
    }

    #[cfg(feature = "bluetooth-test")]
    pub fn set_uuids(&self, uuids: Vec<String>) -> Result<(), Box<dyn Error>> {
        get_inner_and_call_test_func!(self, BluetoothAdapter, set_uuids, uuids)
    }

    #[cfg(feature = "bluetooth-test")]
    pub fn set_modalias(&self, modalias: String) -> Result<(), Box<dyn Error>> {
        get_inner_and_call_test_func!(self, BluetoothAdapter, set_modalias, modalias)
    }

    #[cfg(feature = "bluetooth-test")]
    pub fn get_ad_datas(&self) -> Result<Vec<String>, Box<dyn Error>> {
        get_inner_and_call_test_func!(self, BluetoothAdapter, get_ad_datas)
    }

    #[cfg(feature = "bluetooth-test")]
    pub fn set_ad_datas(&self, ad_datas: Vec<String>) -> Result<(), Box<dyn Error>> {
        get_inner_and_call_test_func!(self, BluetoothAdapter, set_ad_datas, ad_datas)
    }
}
