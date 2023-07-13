/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use std::{hash};
use crate::gpu_cache::{GpuCacheHandle};
use crate::frame_builder::FrameBuildingState;
use crate::gpu_cache::GpuDataRequest;
use crate::intern;
use api::{ComponentTransferFuncType};


pub type FilterDataHandle = intern::Handle<FilterDataIntern>;

#[derive(Debug, Clone, MallocSizeOf, PartialEq)]
#[cfg_attr(feature = "capture", derive(Serialize))]
#[cfg_attr(feature = "replay", derive(Deserialize))]
pub enum SFilterDataComponent {
    Identity,
    Table(Vec<f32>),
    Discrete(Vec<f32>),
    Linear(f32, f32),
    Gamma(f32, f32, f32),
}

impl Eq for SFilterDataComponent {}

impl hash::Hash for SFilterDataComponent {
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        match self {
            SFilterDataComponent::Identity => {
                0.hash(state);
            }
            SFilterDataComponent::Table(values) => {
                1.hash(state);
                values.len().hash(state);
                for val in values {
                    val.to_bits().hash(state);
                }
            }
            SFilterDataComponent::Discrete(values) => {
                2.hash(state);
                values.len().hash(state);
                for val in values {
                    val.to_bits().hash(state);
                }
            }
            SFilterDataComponent::Linear(a, b) => {
                3.hash(state);
                a.to_bits().hash(state);
                b.to_bits().hash(state);
            }
            SFilterDataComponent::Gamma(a, b, c) => {
                4.hash(state);
                a.to_bits().hash(state);
                b.to_bits().hash(state);
                c.to_bits().hash(state);
            }
        }
    }
}

impl SFilterDataComponent {
    pub fn to_int(&self) -> u32 {
        match self {
            SFilterDataComponent::Identity => 0,
            SFilterDataComponent::Table(_) => 1,
            SFilterDataComponent::Discrete(_) => 2,
            SFilterDataComponent::Linear(_, _) => 3,
            SFilterDataComponent::Gamma(_, _, _) => 4,
        }
    }

    pub fn from_functype_values(
        func_type: ComponentTransferFuncType,
        values: &[f32],
    ) -> SFilterDataComponent {
        match func_type {
            ComponentTransferFuncType::Identity => SFilterDataComponent::Identity,
            ComponentTransferFuncType::Table => SFilterDataComponent::Table(values.to_vec()),
            ComponentTransferFuncType::Discrete => SFilterDataComponent::Discrete(values.to_vec()),
            ComponentTransferFuncType::Linear => SFilterDataComponent::Linear(values[0], values[1]),
            ComponentTransferFuncType::Gamma => SFilterDataComponent::Gamma(values[0], values[1], values[2]),
        }
    }
}

#[derive(Debug, Clone, MallocSizeOf, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "capture", derive(Serialize))]
#[cfg_attr(feature = "replay", derive(Deserialize))]
pub struct SFilterData {
    pub r_func: SFilterDataComponent,
    pub g_func: SFilterDataComponent,
    pub b_func: SFilterDataComponent,
    pub a_func: SFilterDataComponent,
}

#[derive(Debug, Clone, MallocSizeOf, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "capture", derive(Serialize))]
#[cfg_attr(feature = "replay", derive(Deserialize))]
pub struct SFilterDataKey {
    pub data: SFilterData,
}

impl intern::InternDebug for SFilterDataKey {}

#[derive(Debug)]
#[cfg_attr(feature = "capture", derive(Serialize))]
#[cfg_attr(feature = "replay", derive(Deserialize))]
#[derive(MallocSizeOf)]
pub struct SFilterDataTemplate {
    pub data: SFilterData,
    pub gpu_cache_handle: GpuCacheHandle,
}

impl From<SFilterDataKey> for SFilterDataTemplate {
    fn from(item: SFilterDataKey) -> Self {
        SFilterDataTemplate {
            data: item.data,
            gpu_cache_handle: GpuCacheHandle::new(),
        }
    }
}

impl SFilterData {
    pub fn is_identity(&self) -> bool {
        self.r_func == SFilterDataComponent::Identity
            && self.g_func == SFilterDataComponent::Identity
            && self.b_func == SFilterDataComponent::Identity
            && self.a_func == SFilterDataComponent::Identity
    }

    pub fn update(&self, mut request: GpuDataRequest) {
        push_component_transfer_data(&self.r_func, &mut request);
        push_component_transfer_data(&self.g_func, &mut request);
        push_component_transfer_data(&self.b_func, &mut request);
        push_component_transfer_data(&self.a_func, &mut request);
        assert!(!self.is_identity());
    }
}

impl SFilterDataTemplate {
    /// Update the GPU cache for a given filter data template. This may be called multiple
    /// times per frame, by each primitive reference that refers to this interned
    /// template. The initial request call to the GPU cache ensures that work is only
    /// done if the cache entry is invalid (due to first use or eviction).
    pub fn update(
        &mut self,
        frame_state: &mut FrameBuildingState,
    ) {
        if let Some(request) = frame_state.gpu_cache.request(&mut self.gpu_cache_handle) {
            self.data.update(request);
        }
    }
}

#[derive(Copy, Clone, Debug, MallocSizeOf)]
#[cfg_attr(any(feature = "serde"), derive(Deserialize, Serialize))]
pub enum FilterDataIntern {}

impl intern::Internable for FilterDataIntern {
    type Key = SFilterDataKey;
    type StoreData = SFilterDataTemplate;
    type InternData = ();
    const PROFILE_COUNTER: usize = crate::profiler::INTERNED_FILTER_DATA;
}

fn push_component_transfer_data(
    func_comp: &SFilterDataComponent,
    request: &mut GpuDataRequest,
) {
    match func_comp {
        SFilterDataComponent::Identity => {}
        SFilterDataComponent::Table(values) |
        SFilterDataComponent::Discrete(values) => {
            // Push a 256 entry lookup table.
            assert!(values.len() > 0);
            for i in 0 .. 64 {
                let mut arr = [0.0 ; 4];
                for j in 0 .. 4 {
                    if (values.len() == 1) || (i == 63 && j == 3) {
                        arr[j] = values[values.len()-1];
                    } else {
                        let c = ((4*i + j) as f32)/255.0;
                        match func_comp {
                            SFilterDataComponent::Table(_) => {
                                let n = (values.len()-1) as f32;
                                let k = (n * c).floor() as u32;
                                let ku = k as usize;
                                assert!(ku < values.len()-1);
                                arr[j] = values[ku] + (c*n - (k as f32)) * (values[ku+1] - values[ku]);
                            }
                            SFilterDataComponent::Discrete(_) => {
                                let n = values.len() as f32;
                                let k = (n * c).floor() as usize;
                                assert!(k < values.len());
                                arr[j] = values[k];
                            }
                            SFilterDataComponent::Identity |
                            SFilterDataComponent::Linear(_,_) |
                            SFilterDataComponent::Gamma(_,_,_) => {
                                unreachable!();
                            }
                        }

                    }
                }

                request.push(arr);
            }
        }
        SFilterDataComponent::Linear(a, b) => {
            request.push([*a, *b, 0.0, 0.0]);
        }
        SFilterDataComponent::Gamma(a, b, c) => {
            request.push([*a, *b, *c, 0.0]);
        }
    }
}
