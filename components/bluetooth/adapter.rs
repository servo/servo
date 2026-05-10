/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use std::error::Error;
use std::sync::Arc;

#[cfg(feature = "bluetooth-test")]
use blurmock::fake_adapter::FakeBluetoothAdapter;
#[cfg(feature = "bluetooth-test")]
use blurmock::fake_device::FakeBluetoothDevice;
#[cfg(feature = "bluetooth-test")]
use blurmock::fake_discovery_session::FakeBluetoothDiscoverySession;
#[cfg(feature = "native-bluetooth")]
use btleplug::api::{Central, CentralState, Manager};
#[cfg(feature = "native-bluetooth")]
use btleplug::platform::{Adapter, Manager as PlatformManager};

use super::bluetooth::{BluetoothDevice, BluetoothDiscoverySession};
use crate::macros::get_inner_and_call_test_func;

#[cfg(feature = "native-bluetooth")]
#[derive(Clone, Debug)]
pub struct BtleplugAdapter {
    adapter: Adapter,
}

#[cfg(feature = "native-bluetooth")]
impl BtleplugAdapter {
    pub async fn new() -> Result<Self, Box<dyn Error>> {
        let manager = PlatformManager::new().await?;
        let adapters = manager.adapters().await?;
        let adapter = adapters
            .into_iter()
            .next()
            .ok_or_else(|| btleplug::Error::NoAdapterAvailable)?;
        Ok(BtleplugAdapter { adapter })
    }

    pub async fn get_address(&self) -> Result<String, Box<dyn Error>> {
        Ok(self.adapter.adapter_info().await?)
    }

    pub async fn is_powered(&self) -> Result<bool, Box<dyn Error>> {
        let state = self.adapter.adapter_state().await?;
        Ok(state == CentralState::PoweredOn)
    }

    pub async fn get_devices(&self) -> Result<Vec<BluetoothDevice>, Box<dyn Error>> {
        let peripherals = self.adapter.peripherals().await?;
        Ok(peripherals
            .into_iter()
            .map(|p| BluetoothDevice::Btleplug(super::bluetooth::BtleplugDevice { peripheral: p }))
            .collect())
    }

    pub fn create_discovery_session(&self) -> Result<BluetoothDiscoverySession, Box<dyn Error>> {
        Ok(BluetoothDiscoverySession::Btleplug(
            super::bluetooth::BtleplugDiscoverySession {
                adapter: self.adapter.clone(),
            },
        ))
    }
}

#[derive(Clone, Debug)]
pub enum BluetoothAdapter {
    #[cfg(feature = "native-bluetooth")]
    Btleplug(BtleplugAdapter),
    #[cfg(feature = "bluetooth-test")]
    Mock(Arc<FakeBluetoothAdapter>),
}

impl BluetoothAdapter {
    #[cfg(feature = "native-bluetooth")]
    pub async fn new() -> Result<BluetoothAdapter, Box<dyn Error>> {
        Ok(Self::Btleplug(BtleplugAdapter::new().await?))
    }

    #[cfg(not(feature = "native-bluetooth"))]
    pub fn new() -> Result<BluetoothAdapter, Box<dyn Error>> {
        Err(Box::from("Bluetooth not supported on this platform"))
    }

    #[cfg(feature = "bluetooth-test")]
    pub fn new_mock() -> Result<BluetoothAdapter, Box<dyn Error>> {
        Ok(Self::Mock(FakeBluetoothAdapter::new_empty()))
    }

    pub async fn get_address(&self) -> Result<String, Box<dyn Error>> {
        match self {
            #[cfg(feature = "native-bluetooth")]
            Self::Btleplug(inner) => inner.get_address().await,
            #[cfg(feature = "bluetooth-test")]
            Self::Mock(inner) => inner.get_address(),
        }
    }

    pub async fn is_powered(&self) -> Result<bool, Box<dyn Error>> {
        match self {
            #[cfg(feature = "native-bluetooth")]
            Self::Btleplug(inner) => inner.is_powered().await,
            #[cfg(feature = "bluetooth-test")]
            Self::Mock(inner) => inner.is_powered(),
        }
    }

    pub async fn get_devices(&self) -> Result<Vec<BluetoothDevice>, Box<dyn Error>> {
        match self {
            #[cfg(feature = "native-bluetooth")]
            Self::Btleplug(inner) => inner.get_devices().await,
            #[cfg(feature = "bluetooth-test")]
            Self::Mock(inner) => {
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

    pub fn create_discovery_session(&self) -> Result<BluetoothDiscoverySession, Box<dyn Error>> {
        match self {
            #[cfg(feature = "native-bluetooth")]
            Self::Btleplug(inner) => inner.create_discovery_session(),
            #[cfg(feature = "bluetooth-test")]
            Self::Mock(inner) => Ok(BluetoothDiscoverySession::Mock(Arc::new(
                FakeBluetoothDiscoverySession::create_session(inner.clone())?,
            ))),
        }
    }

    pub fn create_mock_device(&self, _device: String) -> Result<BluetoothDevice, Box<dyn Error>> {
        match self {
            #[cfg(feature = "bluetooth-test")]
            Self::Mock(inner) => Ok(BluetoothDevice::Mock(FakeBluetoothDevice::new_empty(
                inner.clone(),
                _device,
            ))),
            #[cfg(feature = "native-bluetooth")]
            _ => Err(Box::from("Test functions not supported on real devices")),
        }
    }

    #[cfg(feature = "bluetooth-test")]
    pub fn set_name(&self, name: String) -> Result<(), Box<dyn Error>> {
        get_inner_and_call_test_func!(self, BluetoothAdapter, set_name, name)
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
}
