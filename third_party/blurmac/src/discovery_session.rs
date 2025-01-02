// Copyright (c) 2017 Akos Kiss.
//
// Licensed under the BSD 3-Clause License
// <LICENSE.md or https://opensource.org/licenses/BSD-3-Clause>.
// This file may not be copied, modified, or distributed except
// according to those terms.

use std::error::Error;
use std::sync::Arc;

use adapter::BluetoothAdapter;

#[derive(Clone, Debug)]
pub struct BluetoothDiscoverySession {
    // pub(crate) adapter: Arc<BluetoothAdapter>,
}

impl BluetoothDiscoverySession {
    pub fn create_session(
        _adapter: Arc<BluetoothAdapter>,
    ) -> Result<BluetoothDiscoverySession, Box<dyn Error>> {
        trace!("BluetoothDiscoverySession::create_session");
        Ok(BluetoothDiscoverySession {
            // adapter: adapter.clone()
        })
    }

    pub fn start_discovery(&self) -> Result<(), Box<dyn Error>> {
        trace!("BluetoothDiscoverySession::start_discovery");
        // NOTE: discovery is started by BluetoothAdapter::new to allow devices to pop up
        Ok(())
    }

    pub fn stop_discovery(&self) -> Result<(), Box<dyn Error>> {
        trace!("BluetoothDiscoverySession::stop_discovery");
        // NOTE: discovery is only stopped when BluetoothAdapter is dropped
        Ok(())
    }
}
