/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::f32::consts::PI;

use euclid::default::Vector3D;

use crate::block::{Block, Chunk, FRAMES_PER_BLOCK, Tick};
use crate::node::{AudioNodeEngine, AudioNodeMessage, AudioNodeType, BlockInfo, ChannelInfo};
use crate::param::{Param, ParamDir, ParamType};

// .normalize(), but it takes into account zero vectors
pub fn normalize_zero(v: Vector3D<f32>) -> Vector3D<f32> {
    let len = v.length();
    if len == 0. { v } else { v / len }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum PanningModel {
    EqualPower,
    HRTF,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum DistanceModel {
    Linear,
    Inverse,
    Exponential,
}

#[derive(Copy, Clone, Debug)]
pub struct PannerNodeOptions {
    pub panning_model: PanningModel,
    pub distance_model: DistanceModel,
    pub position_x: f32,
    pub position_y: f32,
    pub position_z: f32,
    pub orientation_x: f32,
    pub orientation_y: f32,
    pub orientation_z: f32,
    pub ref_distance: f64,
    pub max_distance: f64,
    pub rolloff_factor: f64,
    pub cone_inner_angle: f64,
    pub cone_outer_angle: f64,
    pub cone_outer_gain: f64,
}

pub enum PannerNodeMessage {
    SetPanningModel(PanningModel),
    SetDistanceModel(DistanceModel),
    SetRefDistance(f64),
    SetMaxDistance(f64),
    SetRolloff(f64),
    SetConeInner(f64),
    SetConeOuter(f64),
    SetConeGain(f64),
}

impl Default for PannerNodeOptions {
    fn default() -> Self {
        PannerNodeOptions {
            panning_model: PanningModel::EqualPower,
            distance_model: DistanceModel::Inverse,
            position_x: 0.,
            position_y: 0.,
            position_z: 0.,
            orientation_x: 1.,
            orientation_y: 0.,
            orientation_z: 0.,
            ref_distance: 1.,
            max_distance: 10000.,
            rolloff_factor: 1.,
            cone_inner_angle: 360.,
            cone_outer_angle: 360.,
            cone_outer_gain: 0.,
        }
    }
}

#[derive(AudioNodeCommon)]
pub(crate) struct PannerNode {
    channel_info: ChannelInfo,
    panning_model: PanningModel,
    distance_model: DistanceModel,
    position_x: Param,
    position_y: Param,
    position_z: Param,
    orientation_x: Param,
    orientation_y: Param,
    orientation_z: Param,
    ref_distance: f64,
    max_distance: f64,
    rolloff_factor: f64,
    cone_inner_angle: f64,
    cone_outer_angle: f64,
    cone_outer_gain: f64,
    listener_data: Option<Block>,
}

impl PannerNode {
    pub fn new(options: PannerNodeOptions, channel_info: ChannelInfo) -> Self {
        if options.panning_model == PanningModel::HRTF {
            log::warn!("HRTF requested but not supported")
        }
        Self {
            channel_info,
            panning_model: options.panning_model,
            distance_model: options.distance_model,
            position_x: Param::new(options.position_x),
            position_y: Param::new(options.position_y),
            position_z: Param::new(options.position_z),
            orientation_x: Param::new(options.orientation_x),
            orientation_y: Param::new(options.orientation_y),
            orientation_z: Param::new(options.orientation_z),
            ref_distance: options.ref_distance,
            max_distance: options.max_distance,
            rolloff_factor: options.rolloff_factor,
            cone_inner_angle: options.cone_inner_angle,
            cone_outer_angle: options.cone_outer_angle,
            cone_outer_gain: options.cone_outer_gain,
            listener_data: None,
        }
    }

    pub fn update_parameters(&mut self, info: &BlockInfo, tick: Tick) -> bool {
        let mut changed = self.position_x.update(info, tick);
        changed |= self.position_y.update(info, tick);
        changed |= self.position_z.update(info, tick);
        changed |= self.orientation_x.update(info, tick);
        changed |= self.orientation_y.update(info, tick);
        changed |= self.orientation_z.update(info, tick);
        changed
    }

    /// Computes azimuth, elevation, and distance of source with respect to a
    /// given AudioListener's position, forward, and up vectors
    /// in degrees
    ///
    /// <https://webaudio.github.io/web-audio-api/#azimuth-elevation>
    /// <https://webaudio.github.io/web-audio-api/#Spatialization-distance-effects>
    fn azimuth_elevation_distance(
        &self,
        listener: (Vector3D<f32>, Vector3D<f32>, Vector3D<f32>),
    ) -> (f32, f32, f64) {
        let (listener_position, listener_forward, listener_up) = listener;
        let source_position = Vector3D::new(
            self.position_x.value(),
            self.position_y.value(),
            self.position_z.value(),
        );

        // degenerate case
        if source_position == listener_position {
            return (0., 0., 0.);
        }

        let diff = source_position - listener_position;
        let distance = diff.length();
        let source_listener = normalize_zero(diff);
        let listener_right = listener_forward.cross(listener_up);
        let listener_right_norm = normalize_zero(listener_right);
        let listener_forward_norm = normalize_zero(listener_forward);

        let up = listener_right_norm.cross(listener_forward_norm);

        let up_projection = source_listener.dot(up);
        let projected_source = normalize_zero(source_listener - up * up_projection);
        let mut azimuth = 180. * projected_source.dot(listener_right_norm).acos() / PI;

        let front_back = projected_source.dot(listener_forward_norm);
        if front_back < 0. {
            azimuth = 360. - azimuth;
        }
        if (0. ..=270.).contains(&azimuth) {
            azimuth = 90. - azimuth;
        } else {
            azimuth = 450. - azimuth;
        }

        let mut elevation = 90. - 180. * source_listener.dot(up).acos() / PI;

        if elevation > 90. {
            elevation = 180. - elevation;
        } else if elevation < -90. {
            elevation = -180. - elevation;
        }

        (azimuth, elevation, distance as f64)
    }

    /// <https://webaudio.github.io/web-audio-api/#Spatialization-sound-cones>
    fn cone_gain(&self, listener: (Vector3D<f32>, Vector3D<f32>, Vector3D<f32>)) -> f64 {
        let (listener_position, _, _) = listener;
        let source_position = Vector3D::new(
            self.position_x.value(),
            self.position_y.value(),
            self.position_z.value(),
        );
        let source_orientation = Vector3D::new(
            self.orientation_x.value(),
            self.orientation_y.value(),
            self.orientation_z.value(),
        );

        if source_orientation == Vector3D::zero() ||
            (self.cone_inner_angle == 360. && self.cone_outer_angle == 360.)
        {
            return 0.;
        }

        let normalized_source_orientation = normalize_zero(source_orientation);

        let source_to_listener = normalize_zero(source_position - listener_position);
        // Angle between the source orientation vector and the source-listener vector
        let angle = 180. * source_to_listener.dot(normalized_source_orientation).acos() / PI;
        let abs_angle = angle.abs() as f64;

        // Divide by 2 here since API is entire angle (not half-angle)
        let abs_inner_angle = self.cone_inner_angle.abs() / 2.;
        let abs_outer_angle = self.cone_outer_angle.abs() / 2.;

        if abs_angle < abs_inner_angle {
            // no attenuation
            1.
        } else if abs_angle >= abs_outer_angle {
            // max attenuation
            self.cone_outer_gain
        } else {
            // gain changes linearly from 1 to cone_outer_gain
            // as we go from inner to outer
            let x = (abs_angle - abs_inner_angle) / (abs_outer_angle - abs_inner_angle);
            (1. - x) + self.cone_outer_gain * x
        }
    }

    fn linear_distance(&self, mut distance: f64, rolloff_factor: f64) -> f64 {
        if distance > self.max_distance {
            distance = self.max_distance;
        }
        if distance < self.ref_distance {
            distance = self.ref_distance;
        }
        let denom = self.max_distance - self.ref_distance;
        1. - rolloff_factor * (distance - self.ref_distance) / denom
    }

    fn inverse_distance(&self, mut distance: f64, rolloff_factor: f64) -> f64 {
        if distance < self.ref_distance {
            distance = self.ref_distance;
        }
        let denom = self.ref_distance + rolloff_factor * (distance - self.ref_distance);
        self.ref_distance / denom
    }

    fn exponential_distance(&self, mut distance: f64, rolloff_factor: f64) -> f64 {
        if distance < self.ref_distance {
            distance = self.ref_distance;
        }

        (distance / self.ref_distance).powf(-rolloff_factor)
    }

    fn distance_gain_fn(&self) -> fn(&Self, f64, f64) -> f64 {
        match self.distance_model {
            DistanceModel::Linear => |x, d, r| x.linear_distance(d, r),
            DistanceModel::Inverse => |x, d, r| x.inverse_distance(d, r),
            DistanceModel::Exponential => |x, d, r| x.exponential_distance(d, r),
        }
    }
}

impl AudioNodeEngine for PannerNode {
    fn node_type(&self) -> AudioNodeType {
        AudioNodeType::PannerNode
    }

    fn process(&mut self, mut inputs: Chunk, info: &BlockInfo) -> Chunk {
        debug_assert!(inputs.len() == 1);

        let listener_data = if let Some(listener_data) = self.listener_data.take() {
            listener_data
        } else {
            return inputs;
        };

        // We clamp this early
        let rolloff_factor =
            if self.distance_model == DistanceModel::Linear && self.rolloff_factor > 1. {
                1.
            } else {
                self.rolloff_factor
            };

        {
            let block = &mut inputs.blocks[0];

            block.explicit_repeat();

            let mono = if block.chan_count() == 1 {
                block.resize_silence(2);
                true
            } else {
                debug_assert!(block.chan_count() == 2);
                false
            };

            let distance_gain_fn = self.distance_gain_fn();

            if self.panning_model == PanningModel::EqualPower {
                let (l, r) = block.data_mut().split_at_mut(FRAMES_PER_BLOCK.0 as usize);
                for frame in 0..FRAMES_PER_BLOCK.0 {
                    let frame = Tick(frame);
                    self.update_parameters(info, frame);
                    let data = listener_data.listener_data(frame);
                    let (mut azimuth, _elev, dist) = self.azimuth_elevation_distance(data);
                    let distance_gain = distance_gain_fn(self, dist, rolloff_factor);
                    let cone_gain = self.cone_gain(data);

                    // https://webaudio.github.io/web-audio-api/#Spatialization-equal-power-panning

                    // clamp to [-180, 180], then wrap to [-90, 90]
                    azimuth = azimuth.clamp(-180., 180.);
                    if azimuth < -90. {
                        azimuth = -180. - azimuth;
                    } else if azimuth > 90. {
                        azimuth = 180. - azimuth;
                    }

                    let x = if mono {
                        (azimuth + 90.) / 180.
                    } else if azimuth <= 0. {
                        (azimuth + 90.) / 90.
                    } else {
                        azimuth / 90.
                    };
                    let x = x * PI / 2.;

                    let mut gain_l = x.cos();
                    let mut gain_r = x.sin();
                    // 9. * PI / 2 is often slightly negative, clamp
                    if gain_l <= 0. {
                        gain_l = 0.
                    }
                    if gain_r <= 0. {
                        gain_r = 0.;
                    }

                    let index = frame.0 as usize;
                    if mono {
                        let input = l[index];
                        l[index] = input * gain_l;
                        r[index] = input * gain_r;
                    } else if azimuth <= 0. {
                        l[index] += r[index] * gain_l;
                        r[index] *= gain_r;
                    } else {
                        r[index] += l[index] * gain_r;
                        l[index] *= gain_l;
                    }
                    l[index] = l[index] * distance_gain as f32 * cone_gain as f32;
                    r[index] = r[index] * distance_gain as f32 * cone_gain as f32;
                }
            }
        }

        inputs
    }

    fn input_count(&self) -> u32 {
        1
    }

    fn get_param(&mut self, id: ParamType) -> &mut Param {
        match id {
            ParamType::Position(ParamDir::X) => &mut self.position_x,
            ParamType::Position(ParamDir::Y) => &mut self.position_y,
            ParamType::Position(ParamDir::Z) => &mut self.position_z,
            ParamType::Orientation(ParamDir::X) => &mut self.orientation_x,
            ParamType::Orientation(ParamDir::Y) => &mut self.orientation_y,
            ParamType::Orientation(ParamDir::Z) => &mut self.orientation_z,
            _ => panic!("Unknown param {:?} for PannerNode", id),
        }
    }

    fn set_listenerdata(&mut self, data: Block) {
        self.listener_data = Some(data);
    }

    fn message_specific(&mut self, message: AudioNodeMessage, _sample_rate: f32) {
        if let AudioNodeMessage::PannerNode(p) = message {
            match p {
                PannerNodeMessage::SetPanningModel(p) => {
                    if p == PanningModel::HRTF {
                        log::warn!("HRTF requested but not supported");
                    }
                    self.panning_model = p;
                },
                PannerNodeMessage::SetDistanceModel(d) => self.distance_model = d,
                PannerNodeMessage::SetRefDistance(val) => self.ref_distance = val,
                PannerNodeMessage::SetMaxDistance(val) => self.max_distance = val,
                PannerNodeMessage::SetRolloff(val) => self.rolloff_factor = val,
                PannerNodeMessage::SetConeInner(val) => self.cone_inner_angle = val,
                PannerNodeMessage::SetConeOuter(val) => self.cone_outer_angle = val,
                PannerNodeMessage::SetConeGain(val) => self.cone_outer_gain = val,
            }
        }
    }
}
