/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::ops::{Deref, DerefMut};

use euclid::default::Size2D;
use ipc_channel::ipc::IpcSharedMemory;
use pixels::Multiply;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub enum PixelFormat {
    #[default]
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

impl Default for AlphaMode {
    fn default() -> Self {
        Self::Transparent {
            premultiplied: true,
        }
    }
}

impl AlphaMode {
    pub const fn is_premultiplied(&self) -> bool {
        match self {
            AlphaMode::Opaque => true,
            AlphaMode::AsOpaque { premultiplied } => *premultiplied,
            AlphaMode::Transparent { premultiplied } => *premultiplied,
        }
    }

    pub const fn is_opaque(&self) -> bool {
        matches!(self, AlphaMode::Opaque | AlphaMode::AsOpaque { .. })
    }
}

#[derive(Debug)]
pub enum Data {
    // TODO: https://github.com/servo/servo/issues/36594
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

/// Represents image bitmap with metadata, usually as snapshot of canvas
///
/// This allows us to hold off conversions (BGRA <-> RGBA, (un)premultiply)
/// to when/if they are actually needed (WebGL/WebGPU can load both BGRA and RGBA).
///
/// Inspired by snapshot for concept in WebGPU spec:
/// <https://gpuweb.github.io/gpuweb/#abstract-opdef-get-a-copy-of-the-image-contents-of-a-context>
#[derive(Debug, Deserialize, Serialize)]
pub struct Snapshot<T = Data> {
    size: Size2D<u64>,
    /// internal data (can be any format it will be converted on use if needed)
    data: T,
    /// RGBA/BGRA (reflect internal data)
    format: PixelFormat,
    /// How to treat alpha channel
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

    pub const fn is_premultiplied(&self) -> bool {
        self.alpha_mode().is_premultiplied()
    }

    pub const fn is_opaque(&self) -> bool {
        self.alpha_mode().is_opaque()
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

    /// Returns snapshot with provided size that is black transparent alpha
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

    pub fn from_shared_memory(
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

    // TODO: https://github.com/servo/servo/issues/36594
    /*
    /// # Safety
    ///
    /// This is safe if data is owned by this process only
    /// (ownership is transferred on send)
    pub unsafe fn from_shared_memory(
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

    /// Convert inner data of snapshot to target format and alpha mode.
    /// If data is already in target format and alpha mode no work will be done.
    pub fn transform(&mut self, target_alpha_mode: AlphaMode, target_format: PixelFormat) {
        let swap_rb = target_format != self.format;
        let multiply = match (self.alpha_mode, target_alpha_mode) {
            (AlphaMode::Opaque, _) => Multiply::None,
            (alpha_mode, AlphaMode::Opaque) => {
                if alpha_mode.is_premultiplied() {
                    Multiply::UnMultiply
                } else {
                    Multiply::None
                }
            },
            (
                AlphaMode::Transparent { premultiplied } | AlphaMode::AsOpaque { premultiplied },
                AlphaMode::Transparent {
                    premultiplied: target_premultiplied,
                } |
                AlphaMode::AsOpaque {
                    premultiplied: target_premultiplied,
                },
            ) => {
                if premultiplied == target_premultiplied {
                    Multiply::None
                } else if target_premultiplied {
                    Multiply::PreMultiply
                } else {
                    Multiply::UnMultiply
                }
            },
        };
        let clear_alpha = !matches!(self.alpha_mode, AlphaMode::Opaque) &&
            matches!(target_alpha_mode, AlphaMode::Opaque);
        pixels::transform_inplace(self.data.deref_mut(), multiply, swap_rb, clear_alpha);
        self.alpha_mode = target_alpha_mode;
        self.format = target_format;
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
    // TODO: https://github.com/servo/servo/issues/36594
    /*
    /// # Safety
    ///
    /// This is safe if data is owned by this process only
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
