/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::ops::{Deref, DerefMut};

use euclid::default::Size2D;
use ipc_channel::ipc::IpcSharedMemory;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum PixelFormat {
    RGBA,
    BGRA,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum AlphaMode {
    /// Internal data is opaque (alpha is cleared to 1)
    Opaque,
    /// Internal data should be threated as opaque (does not mean it actually is)
    AsOpaque { premultiplied: bool },
    /// Data is not opaque
    Transparent { premultiplied: bool },
}

impl AlphaMode {
    pub const fn premultiplied(&self) -> bool {
        match self {
            AlphaMode::Opaque => true,
            AlphaMode::AsOpaque { premultiplied } => *premultiplied,
            AlphaMode::Transparent { premultiplied } => *premultiplied,
        }
    }

    pub const fn opaque(&self) -> bool {
        matches!(self, AlphaMode::Opaque | AlphaMode::AsOpaque { .. })
    }
}

#[derive(Debug)]
pub enum Data {
    //IPC(IpcSharedMemory),
    Owned(Vec<u8>),
}

impl Deref for Data {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        match &self {
            //Data::IPC(ipc_shared_memory) => ipc_shared_memory,
            Data::Owned(items) => items,
        }
    }
}

impl DerefMut for Data {
    fn deref_mut(&mut self) -> &mut Self::Target {
        match self {
            //Data::IPC(ipc_shared_memory) => unsafe { ipc_shared_memory.deref_mut() },
            Data::Owned(items) => items,
        }
    }
}

pub type IpcSnapshot = Snapshot<IpcSharedMemory>;

#[derive(Debug, Deserialize, Serialize)]
pub struct Snapshot<T = Data> {
    size: Size2D<u64>,
    /// internal data (can be any format it will be converted on use if needed)
    data: T,
    /// RGBA/BGRA (reflect internal data)
    format: PixelFormat,
    /// How to threat alpha channel
    alpha_mode: AlphaMode,
}

impl<T> Snapshot<T> {
    pub const fn size(&self) -> Size2D<u64> {
        self.size
    }

    pub const fn format(&self) -> PixelFormat {
        self.format
    }

    pub const fn alpha_mode(&self) -> AlphaMode {
        self.alpha_mode
    }

    pub const fn premultiplied(&self) -> bool {
        self.alpha_mode().premultiplied()
    }

    pub const fn opaque(&self) -> bool {
        self.alpha_mode().opaque()
    }
}

impl Snapshot<Data> {
    pub fn empty() -> Self {
        Self {
            size: Size2D::zero(),
            data: Data::Owned(vec![]),
            format: PixelFormat::RGBA,
            alpha_mode: AlphaMode::Transparent {
                premultiplied: true,
            },
        }
    }

    pub fn cleared(size: Size2D<u64>) -> Self {
        Self {
            size,
            data: Data::Owned(vec![0; size.area() as usize * 4]),
            format: PixelFormat::RGBA,
            alpha_mode: AlphaMode::Transparent {
                premultiplied: true,
            },
        }
    }

    pub fn new(
        size: Size2D<u64>,
        format: PixelFormat,
        alpha_mode: AlphaMode,
        data: Vec<u8>,
    ) -> Self {
        Self {
            size,
            data: Data::Owned(data),
            format,
            alpha_mode,
        }
    }

    pub fn from_vec(
        size: Size2D<u64>,
        format: PixelFormat,
        alpha_mode: AlphaMode,
        data: Vec<u8>,
    ) -> Self {
        Self {
            size,
            data: Data::Owned(data),
            format,
            alpha_mode,
        }
    }

    pub fn from_ism(
        size: Size2D<u64>,
        format: PixelFormat,
        alpha_mode: AlphaMode,
        ism: IpcSharedMemory,
    ) -> Self {
        Self {
            size,
            data: Data::Owned(ism.to_vec()),
            format,
            alpha_mode,
        }
    }

    /*
    /// # Safety
    ///
    /// This is safe is data is owned by this proces only
    /// (ownership is transferred on send)
    pub unsafe fn from_ism(
        size: Size2D<u64>,
        format: PixelFormat,
        alpha_mode: AlphaMode,
        ism: IpcSharedMemory,
    ) -> Self {
        Self {
            size,
            data: Data::IPC(ism),
            format,
            alpha_mode,
        }
    }
    */

    pub fn data(&self) -> &[u8] {
        &self.data
    }

    pub fn unpremultiply(&mut self, target_format: PixelFormat) {
        if self.alpha_mode.premultiplied() {
            let swap_rb = target_format != self.format;
            if swap_rb {
                pixels::unmultiply_inplace::<true>(self.data.deref_mut());
            } else {
                pixels::unmultiply_inplace::<false>(self.data.deref_mut());
            }
        }
    }

    pub fn as_ipc(self) -> Snapshot<IpcSharedMemory> {
        let Snapshot {
            size,
            data,
            format,
            alpha_mode,
        } = self;
        let data = match data {
            //Data::IPC(ipc_shared_memory) => ipc_shared_memory,
            Data::Owned(items) => IpcSharedMemory::from_bytes(&items),
        };
        Snapshot {
            size,
            data,
            format,
            alpha_mode,
        }
    }

    pub fn to_vec(self) -> Vec<u8> {
        match self.data {
            Data::Owned(data) => data,
        }
    }
}

impl Snapshot<IpcSharedMemory> {
    /*
    /// # Safety
    ///
    /// This is safe is data is owned by this proces only
    /// (ownership is transferred on send)
    pub unsafe fn to_data(self) -> Snapshot<Data> {
        let Snapshot {
            size,
            data,
            format,
            alpha_mode,
        } = self;
        Snapshot {
            size,
            data: Data::IPC(data),
            format,
            alpha_mode,
        }
    }
    */
    pub fn to_owned(self) -> Snapshot<Data> {
        let Snapshot {
            size,
            data,
            format,
            alpha_mode,
        } = self;
        Snapshot {
            size,
            data: Data::Owned(data.to_vec()),
            format,
            alpha_mode,
        }
    }

    pub fn data(&self) -> &[u8] {
        &self.data
    }

    pub fn to_ipc_shared_memory(self) -> IpcSharedMemory {
        self.data
    }
}
