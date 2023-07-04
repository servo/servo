/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#![cfg(windows)]

extern crate gleam;
extern crate mozangle;
extern crate winapi;

use com::{ComPtr, CheckHResult, as_ptr};
use std::ptr;
use std::rc::Rc;
use winapi::shared::dxgi1_2::DXGI_SWAP_CHAIN_DESC1;
use winapi::shared::dxgi1_2::IDXGIFactory2;
use winapi::shared::minwindef::{TRUE, FALSE};
use winapi::shared::windef::HWND;
use winapi::um::d3d11::ID3D11Device;
use winapi::um::dcomp::IDCompositionDevice;
use winapi::um::dcomp::IDCompositionTarget;
use winapi::um::dcomp::IDCompositionVisual;

mod com;
mod egl;

pub struct DirectComposition {
    d3d_device: ComPtr<ID3D11Device>,
    dxgi_factory: ComPtr<IDXGIFactory2>,

    egl: Rc<egl::SharedEglThings>,
    pub gleam: Rc<dyn gleam::gl::Gl>,

    composition_device: ComPtr<IDCompositionDevice>,
    root_visual: ComPtr<IDCompositionVisual>,

    #[allow(unused)]  // Needs to be kept alive
    composition_target: ComPtr<IDCompositionTarget>,
}

impl DirectComposition {
    /// Initialize DirectComposition in the given window
    ///
    /// # Safety
    ///
    /// `hwnd` must be a valid handle to a window.
    pub unsafe fn new(hwnd: HWND) -> Self {
        let d3d_device = ComPtr::new_with(|ptr_ptr| winapi::um::d3d11::D3D11CreateDevice(
            ptr::null_mut(),
            winapi::um::d3dcommon::D3D_DRIVER_TYPE_HARDWARE,
            ptr::null_mut(),
            winapi::um::d3d11::D3D11_CREATE_DEVICE_BGRA_SUPPORT |
            if cfg!(debug_assertions) {
                winapi::um::d3d11::D3D11_CREATE_DEVICE_DEBUG
            } else {
                0
            },
            ptr::null_mut(),
            0,
            winapi::um::d3d11::D3D11_SDK_VERSION,
            ptr_ptr,
            &mut 0,
            ptr::null_mut(),
        ));

        let egl = egl::SharedEglThings::new(d3d_device.as_raw());
        let gleam = gleam::gl::GlesFns::load_with(egl::get_proc_address);

        let dxgi_device = d3d_device.cast::<winapi::shared::dxgi::IDXGIDevice>();

        // https://msdn.microsoft.com/en-us/library/windows/desktop/hh404556(v=vs.85).aspx#code-snippet-1
        // “Because you can create a Direct3D device without creating a swap chain,
        //  you might need to retrieve the factory that is used to create the device
        //  in order to create a swap chain.”
        let adapter = ComPtr::new_with(|ptr_ptr| dxgi_device.GetAdapter(ptr_ptr));
        let dxgi_factory = ComPtr::<IDXGIFactory2>::new_with_uuid(|uuid, ptr_ptr| {
            adapter.GetParent(uuid, ptr_ptr)
        });

        // Create the DirectComposition device object.
        let composition_device = ComPtr::<IDCompositionDevice>::new_with_uuid(|uuid, ptr_ptr| {
            winapi::um::dcomp::DCompositionCreateDevice(&*dxgi_device, uuid, ptr_ptr)
        });

        // Create the composition target object based on the
        // specified application window.
        let composition_target = ComPtr::new_with(|ptr_ptr| {
            composition_device.CreateTargetForHwnd(hwnd, TRUE, ptr_ptr)
        });

        let root_visual = ComPtr::new_with(|ptr_ptr| composition_device.CreateVisual(ptr_ptr));
        composition_target.SetRoot(&*root_visual).check_hresult();

        DirectComposition {
            d3d_device, dxgi_factory,
            egl, gleam,
            composition_device, composition_target, root_visual,
        }
    }

    /// Execute changes to the DirectComposition scene.
    pub fn commit(&self) {
        unsafe {
            self.composition_device.Commit().check_hresult()
        }
    }

    pub fn create_angle_visual(&self, width: u32, height: u32) -> AngleVisual {
        unsafe {
            let desc = DXGI_SWAP_CHAIN_DESC1 {
                Width: width,
                Height: height,
                Format: winapi::shared::dxgiformat::DXGI_FORMAT_B8G8R8A8_UNORM,
                Stereo: FALSE,
                SampleDesc: winapi::shared::dxgitype::DXGI_SAMPLE_DESC {
                    Count: 1,
                    Quality: 0,
                },
                BufferUsage: winapi::shared::dxgitype::DXGI_USAGE_RENDER_TARGET_OUTPUT,
                BufferCount: 2,
                Scaling: winapi::shared::dxgi1_2::DXGI_SCALING_STRETCH,
                SwapEffect: winapi::shared::dxgi::DXGI_SWAP_EFFECT_FLIP_SEQUENTIAL,
                AlphaMode:  winapi::shared::dxgi1_2::DXGI_ALPHA_MODE_PREMULTIPLIED,
                Flags: 0,
            };
            let swap_chain = ComPtr::<winapi::shared::dxgi1_2::IDXGISwapChain1>::new_with(|ptr_ptr| {
                self.dxgi_factory.CreateSwapChainForComposition(
                    as_ptr(&self.d3d_device),
                    &desc,
                    ptr::null_mut(),
                    ptr_ptr,
                )
            });
            let back_buffer = ComPtr::<winapi::um::d3d11::ID3D11Texture2D>::new_with_uuid(|uuid, ptr_ptr| {
                swap_chain.GetBuffer(0, uuid, ptr_ptr)
            });
            let egl = egl::PerVisualEglThings::new(self.egl.clone(), &*back_buffer, width, height);
            let gleam = self.gleam.clone();

            let visual = ComPtr::new_with(|ptr_ptr| self.composition_device.CreateVisual(ptr_ptr));
            visual.SetContent(&*****swap_chain).check_hresult();
            self.root_visual.AddVisual(&*visual, FALSE, ptr::null_mut()).check_hresult();

            AngleVisual { visual, swap_chain, egl, gleam }
        }
    }
}

/// A DirectComposition "visual" configured for rendering with Direct3D.
pub struct AngleVisual {
    visual: ComPtr<IDCompositionVisual>,
    swap_chain: ComPtr<winapi::shared::dxgi1_2::IDXGISwapChain1>,
    egl: egl::PerVisualEglThings,
    pub gleam: Rc<dyn gleam::gl::Gl>,
}

impl AngleVisual {
    pub fn set_offset_x(&self, offset_x: f32) {
        unsafe {
            self.visual.SetOffsetX_1(offset_x).check_hresult()
        }
    }

    pub fn set_offset_y(&self, offset_y: f32) {
        unsafe {
            self.visual.SetOffsetY_1(offset_y).check_hresult()
        }
    }

    pub fn make_current(&self) {
        self.egl.make_current()
    }

    pub fn present(&self) {
        self.gleam.finish();
        unsafe {
            self.swap_chain.Present(0, 0).check_hresult()
        }
    }
}
