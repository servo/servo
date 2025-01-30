use std::ffi::c_void;
use std::mem::MaybeUninit;

use euclid::RigidTransform3D;
use log::debug;
use openxr::sys::{
    HandJointLocationsEXT, HandJointsLocateInfoEXT, HandTrackingAimStateFB,
    FB_HAND_TRACKING_AIM_EXTENSION_NAME,
};
use openxr::{
    self, Action, ActionSet, Binding, FrameState, Graphics, Hand as HandEnum, HandJoint,
    HandJointLocation, HandTracker, HandTrackingAimFlagsFB, Instance, Path, Posef, Session, Space,
    SpaceLocationFlags, HAND_JOINT_COUNT,
};
use webxr_api::Finger;
use webxr_api::Hand;
use webxr_api::Handedness;
use webxr_api::Input;
use webxr_api::InputFrame;
use webxr_api::InputId;
use webxr_api::InputSource;
use webxr_api::JointFrame;
use webxr_api::Native;
use webxr_api::SelectEvent;
use webxr_api::TargetRayMode;
use webxr_api::Viewer;

use super::interaction_profiles::InteractionProfile;
use super::IDENTITY_POSE;

use crate::ext_string;
use crate::openxr::interaction_profiles::INTERACTION_PROFILES;

/// Number of frames to wait with the menu gesture before
/// opening the menu.
const MENU_GESTURE_SUSTAIN_THRESHOLD: u8 = 60;

/// Helper macro for binding action paths in an interaction profile entry
macro_rules! bind_inputs {
    ($actions:expr, $paths:expr, $hand:expr, $instance:expr, $ret:expr) => {
        $actions.iter().enumerate().for_each(|(i, action)| {
            let action_path = $paths[i];
            if action_path != "" {
                let path = $instance
                    .string_to_path(&format!("/user/hand/{}/input/{}", $hand, action_path))
                    .expect(&format!(
                        "Failed to create path for /user/hand/{}/input/{}",
                        $hand, action_path
                    ));
                let binding = Binding::new(action, path);
                $ret.push(binding);
            }
        });
    };
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
enum ClickState {
    Clicking,
    Done,
}

/// All the information on a single input frame
pub struct Frame {
    pub frame: InputFrame,
    pub select: Option<SelectEvent>,
    pub squeeze: Option<SelectEvent>,
    pub menu_selected: bool,
}

impl ClickState {
    fn update_from_action<G: Graphics>(
        &mut self,
        action: &Action<bool>,
        session: &Session<G>,
        menu_selected: bool,
    ) -> (/* is_active */ bool, Option<SelectEvent>) {
        let click = action.state(session, Path::NULL).unwrap();

        let select_event =
            self.update_from_value(click.current_state, click.is_active, menu_selected);

        (click.is_active, select_event)
    }

    fn update_from_value(
        &mut self,
        current_state: bool,
        is_active: bool,
        menu_selected: bool,
    ) -> Option<SelectEvent> {
        if is_active {
            match (current_state, *self) {
                (_, ClickState::Clicking) if menu_selected => {
                    *self = ClickState::Done;
                    // Cancel the select, we're showing a menu
                    Some(SelectEvent::End)
                }
                (true, ClickState::Done) => {
                    *self = ClickState::Clicking;
                    Some(SelectEvent::Start)
                }
                (false, ClickState::Clicking) => {
                    *self = ClickState::Done;
                    Some(SelectEvent::Select)
                }
                _ => None,
            }
        } else if *self == ClickState::Clicking {
            *self = ClickState::Done;
            // Cancel the select, we lost tracking
            Some(SelectEvent::End)
        } else {
            None
        }
    }
}

pub struct OpenXRInput {
    id: InputId,
    action_aim_pose: Action<Posef>,
    action_aim_space: Space,
    action_grip_pose: Action<Posef>,
    action_grip_space: Space,
    action_click: Action<bool>,
    action_squeeze: Action<bool>,
    handedness: Handedness,
    click_state: ClickState,
    squeeze_state: ClickState,
    menu_gesture_sustain: u8,
    #[allow(unused)]
    hand_tracker: Option<HandTracker>,
    action_buttons_common: Vec<Action<f32>>,
    action_buttons_left: Vec<Action<f32>>,
    action_buttons_right: Vec<Action<f32>>,
    action_axes_common: Vec<Action<f32>>,
    use_alternate_input_source: bool,
}

fn hand_str(h: Handedness) -> &'static str {
    match h {
        Handedness::Right => "right",
        Handedness::Left => "left",
        _ => panic!("We don't support unknown handedness in openxr"),
    }
}

impl OpenXRInput {
    pub fn new<G: Graphics>(
        id: InputId,
        handedness: Handedness,
        action_set: &ActionSet,
        session: &Session<G>,
        needs_hands: bool,
        supported_interaction_profiles: Vec<&'static str>,
    ) -> Self {
        let hand = hand_str(handedness);
        let action_aim_pose: Action<Posef> = action_set
            .create_action(
                &format!("{}_hand_aim", hand),
                &format!("{} hand aim", hand),
                &[],
            )
            .unwrap();
        let action_aim_space = action_aim_pose
            .create_space(session.clone(), Path::NULL, IDENTITY_POSE)
            .unwrap();
        let action_grip_pose: Action<Posef> = action_set
            .create_action(
                &format!("{}_hand_grip", hand),
                &format!("{} hand grip", hand),
                &[],
            )
            .unwrap();
        let action_grip_space = action_grip_pose
            .create_space(session.clone(), Path::NULL, IDENTITY_POSE)
            .unwrap();
        let action_click: Action<bool> = action_set
            .create_action(
                &format!("{}_hand_click", hand),
                &format!("{} hand click", hand),
                &[],
            )
            .unwrap();
        let action_squeeze: Action<bool> = action_set
            .create_action(
                &format!("{}_hand_squeeze", hand),
                &format!("{} hand squeeze", hand),
                &[],
            )
            .unwrap();

        let hand_tracker = if needs_hands {
            let hand = match handedness {
                Handedness::Left => HandEnum::LEFT,
                Handedness::Right => HandEnum::RIGHT,
                _ => panic!("We don't support unknown handedness in openxr"),
            };
            session.create_hand_tracker(hand).ok()
        } else {
            None
        };

        let action_buttons_common: Vec<Action<f32>> = {
            let button1: Action<f32> = action_set
                .create_action(
                    &format!("{}_trigger", hand),
                    &format!("{}_trigger", hand),
                    &[],
                )
                .unwrap();
            let button2: Action<f32> = action_set
                .create_action(&format!("{}_grip", hand), &format!("{}_grip", hand), &[])
                .unwrap();
            let button3: Action<f32> = action_set
                .create_action(
                    &format!("{}_touchpad_click", hand),
                    &format!("{}_touchpad_click", hand),
                    &[],
                )
                .unwrap();
            let button4: Action<f32> = action_set
                .create_action(
                    &format!("{}_thumbstick_click", hand),
                    &format!("{}_thumbstick_click", hand),
                    &[],
                )
                .unwrap();
            vec![button1, button2, button3, button4]
        };

        let action_buttons_left = {
            let button1: Action<f32> = action_set
                .create_action(&format!("{}_x", hand), &format!("{}_x", hand), &[])
                .unwrap();
            let button2: Action<f32> = action_set
                .create_action(&format!("{}_y", hand), &format!("{}_y", hand), &[])
                .unwrap();
            vec![button1, button2]
        };

        let action_buttons_right = {
            let button1: Action<f32> = action_set
                .create_action(&format!("{}_a", hand), &format!("{}_a", hand), &[])
                .unwrap();
            let button2: Action<f32> = action_set
                .create_action(&format!("{}_b", hand), &format!("{}_b", hand), &[])
                .unwrap();
            vec![button1, button2]
        };

        let action_axes_common: Vec<Action<f32>> = {
            let axis1: Action<f32> = action_set
                .create_action(
                    &format!("{}_touchpad_x", hand),
                    &format!("{}_touchpad_x", hand),
                    &[],
                )
                .unwrap();
            let axis2: Action<f32> = action_set
                .create_action(
                    &format!("{}_touchpad_y", hand),
                    &format!("{}_touchpad_y", hand),
                    &[],
                )
                .unwrap();
            let axis3: Action<f32> = action_set
                .create_action(
                    &format!("{}_thumbstick_x", hand),
                    &format!("{}_thumbstick_x", hand),
                    &[],
                )
                .unwrap();
            let axis4: Action<f32> = action_set
                .create_action(
                    &format!("{}_thumbstick_y", hand),
                    &format!("{}_thumbstick_y", hand),
                    &[],
                )
                .unwrap();
            vec![axis1, axis2, axis3, axis4]
        };

        let use_alternate_input_source = supported_interaction_profiles
            .contains(&ext_string!(FB_HAND_TRACKING_AIM_EXTENSION_NAME));

        Self {
            id,
            action_aim_pose,
            action_aim_space,
            action_grip_pose,
            action_grip_space,
            action_click,
            action_squeeze,
            handedness,
            click_state: ClickState::Done,
            squeeze_state: ClickState::Done,
            menu_gesture_sustain: 0,
            hand_tracker,
            action_buttons_common,
            action_axes_common,
            action_buttons_left,
            action_buttons_right,
            use_alternate_input_source,
        }
    }

    pub fn setup_inputs<G: Graphics>(
        instance: &Instance,
        session: &Session<G>,
        needs_hands: bool,
        supported_interaction_profiles: Vec<&'static str>,
    ) -> (ActionSet, Self, Self) {
        let action_set = instance.create_action_set("hands", "Hands", 0).unwrap();
        let right_hand = OpenXRInput::new(
            InputId(0),
            Handedness::Right,
            &action_set,
            &session,
            needs_hands,
            supported_interaction_profiles.clone(),
        );
        let left_hand = OpenXRInput::new(
            InputId(1),
            Handedness::Left,
            &action_set,
            &session,
            needs_hands,
            supported_interaction_profiles.clone(),
        );

        for profile in INTERACTION_PROFILES {
            if let Some(extension_name) = profile.required_extension {
                if !supported_interaction_profiles.contains(&ext_string!(extension_name)) {
                    continue;
                }
            }

            if profile.path.is_empty() {
                continue;
            }

            let select = profile.standard_buttons[0];
            let squeeze = Option::from(profile.standard_buttons[1]).filter(|&s| !s.is_empty());
            let mut bindings = right_hand.get_bindings(instance, select, squeeze, &profile);
            bindings.extend(
                left_hand
                    .get_bindings(instance, select, squeeze, &profile)
                    .into_iter(),
            );

            let path_controller = instance
                .string_to_path(profile.path)
                .expect(format!("Invalid interaction profile path: {}", profile.path).as_str());
            if let Err(_) =
                instance.suggest_interaction_profile_bindings(path_controller, &bindings)
            {
                debug!(
                    "Interaction profile path not available for this runtime: {:?}",
                    profile.path
                );
            }
        }

        session.attach_action_sets(&[&action_set]).unwrap();

        (action_set, right_hand, left_hand)
    }

    fn get_bindings(
        &self,
        instance: &Instance,
        select_name: &str,
        squeeze_name: Option<&str>,
        interaction_profile: &InteractionProfile,
    ) -> Vec<Binding> {
        let hand = hand_str(self.handedness);
        let path_aim_pose = instance
            .string_to_path(&format!("/user/hand/{}/input/aim/pose", hand))
            .expect(&format!(
                "Failed to create path for /user/hand/{}/input/aim/pose",
                hand
            ));
        let binding_aim_pose = Binding::new(&self.action_aim_pose, path_aim_pose);
        let path_grip_pose = instance
            .string_to_path(&format!("/user/hand/{}/input/grip/pose", hand))
            .expect(&format!(
                "Failed to create path for /user/hand/{}/input/grip/pose",
                hand
            ));
        let binding_grip_pose = Binding::new(&self.action_grip_pose, path_grip_pose);
        let path_click = instance
            .string_to_path(&format!("/user/hand/{}/input/{}", hand, select_name))
            .expect(&format!(
                "Failed to create path for /user/hand/{}/input/{}",
                hand, select_name
            ));
        let binding_click = Binding::new(&self.action_click, path_click);

        let mut ret = vec![binding_aim_pose, binding_grip_pose, binding_click];
        if let Some(squeeze_name) = squeeze_name {
            let path_squeeze = instance
                .string_to_path(&format!("/user/hand/{}/input/{}", hand, squeeze_name))
                .expect(&format!(
                    "Failed to create path for /user/hand/{}/input/{}",
                    hand, squeeze_name
                ));
            let binding_squeeze = Binding::new(&self.action_squeeze, path_squeeze);
            ret.push(binding_squeeze);
        }

        bind_inputs!(
            self.action_buttons_common,
            interaction_profile.standard_buttons,
            hand,
            instance,
            ret
        );

        if !interaction_profile.left_buttons.is_empty() && hand == "left" {
            bind_inputs!(
                self.action_buttons_left,
                interaction_profile.left_buttons,
                hand,
                instance,
                ret
            );
        } else if !interaction_profile.right_buttons.is_empty() && hand == "right" {
            bind_inputs!(
                self.action_buttons_right,
                interaction_profile.right_buttons,
                hand,
                instance,
                ret
            );
        }

        bind_inputs!(
            self.action_axes_common,
            interaction_profile.standard_axes,
            hand,
            instance,
            ret
        );

        ret
    }

    pub fn frame<G: Graphics>(
        &mut self,
        session: &Session<G>,
        frame_state: &FrameState,
        base_space: &Space,
        viewer: &RigidTransform3D<f32, Viewer, Native>,
    ) -> Frame {
        use euclid::Vector3D;
        let mut target_ray_origin = pose_for(&self.action_aim_space, frame_state, base_space);

        let grip_origin = pose_for(&self.action_grip_space, frame_state, base_space);

        let mut menu_selected = false;
        // Check if the palm is facing up. This is our "menu" gesture.
        if let Some(grip_origin) = grip_origin {
            // The X axis of the grip is perpendicular to the palm, however its
            // direction is the opposite for each hand
            //
            // We obtain a unit vector pointing out of the palm
            let x_dir = if let Handedness::Left = self.handedness {
                1.0
            } else {
                -1.0
            };
            // Rotate it by the grip to obtain the desired vector
            let grip_x = grip_origin
                .rotation
                .transform_vector3d(Vector3D::new(x_dir, 0.0, 0.0));
            let gaze = viewer
                .rotation
                .transform_vector3d(Vector3D::new(0., 0., 1.));

            // If the angle is close enough to 0, its cosine will be
            // close to 1
            // check if the user's gaze is parallel to the palm
            if gaze.dot(grip_x) > 0.95 {
                let input_relative = (viewer.translation - grip_origin.translation).normalize();
                // if so, check if the user is actually looking at the palm
                if gaze.dot(input_relative) > 0.95 {
                    self.menu_gesture_sustain += 1;
                    if self.menu_gesture_sustain > MENU_GESTURE_SUSTAIN_THRESHOLD {
                        menu_selected = true;
                        self.menu_gesture_sustain = 0;
                    }
                } else {
                    self.menu_gesture_sustain = 0
                }
            } else {
                self.menu_gesture_sustain = 0;
            }
        } else {
            self.menu_gesture_sustain = 0;
        }

        let hand = hand_str(self.handedness);
        let click = self.action_click.state(session, Path::NULL).unwrap();
        let squeeze = self.action_squeeze.state(session, Path::NULL).unwrap();
        let (button_values, buttons_changed) = {
            let mut changed = false;
            let mut values = Vec::<f32>::new();
            let mut sync_buttons = |actions: &Vec<Action<f32>>| {
                let buttons = actions
                    .iter()
                    .map(|action| {
                        let state = action.state(session, Path::NULL).unwrap();
                        changed = changed || state.changed_since_last_sync;
                        state.current_state
                    })
                    .collect::<Vec<f32>>();
                values.extend_from_slice(&buttons);
            };
            sync_buttons(&self.action_buttons_common);
            if hand == "left" {
                sync_buttons(&self.action_buttons_left);
            } else if hand == "right" {
                sync_buttons(&self.action_buttons_right);
            }
            (values, changed)
        };

        let (axis_values, axes_changed) = {
            let mut changed = false;
            let values = self
                .action_axes_common
                .iter()
                .enumerate()
                .map(|(i, action)| {
                    let state = action.state(session, Path::NULL).unwrap();
                    changed = changed || state.changed_since_last_sync;
                    // Invert input from y axes
                    state.current_state * if i % 2 == 1 { -1.0 } else { 1.0 }
                })
                .collect::<Vec<f32>>();
            (values, changed)
        };

        let input_changed = buttons_changed || axes_changed;

        let (click_is_active, mut click_event) = if !self.use_alternate_input_source {
            self.click_state
                .update_from_action(&self.action_click, session, menu_selected)
        } else {
            (true, None)
        };
        let (squeeze_is_active, squeeze_event) =
            self.squeeze_state
                .update_from_action(&self.action_squeeze, session, menu_selected);

        let mut aim_state: Option<HandTrackingAimStateFB> = None;
        let hand = self.hand_tracker.as_ref().and_then(|tracker| {
            locate_hand(
                base_space,
                tracker,
                frame_state,
                self.use_alternate_input_source,
                session,
                &mut aim_state,
            )
        });

        let mut pressed = click_is_active && click.current_state;
        let squeezed = squeeze_is_active && squeeze.current_state;

        if let Some(state) = aim_state {
            target_ray_origin.replace(super::transform(&state.aim_pose));
            let index_pinching = state
                .status
                .intersects(HandTrackingAimFlagsFB::INDEX_PINCHING);
            click_event = self
                .click_state
                .update_from_value(index_pinching, true, menu_selected);
            pressed = index_pinching;
        }

        let input_frame = InputFrame {
            target_ray_origin,
            id: self.id,
            pressed,
            squeezed,
            grip_origin,
            hand,
            button_values,
            axis_values,
            input_changed,
        };

        Frame {
            frame: input_frame,
            select: click_event,
            squeeze: squeeze_event,
            menu_selected,
        }
    }

    pub fn input_source(&self) -> InputSource {
        let hand_support = if self.hand_tracker.is_some() {
            // openxr runtimes must always support all or none joints
            Some(Hand::<()>::default().map(|_, _| Some(())))
        } else {
            None
        };
        InputSource {
            handedness: self.handedness,
            id: self.id,
            target_ray_mode: TargetRayMode::TrackedPointer,
            supports_grip: true,
            profiles: vec![],
            hand_support,
        }
    }
}

fn pose_for(
    action_space: &Space,
    frame_state: &FrameState,
    base_space: &Space,
) -> Option<RigidTransform3D<f32, Input, Native>> {
    let location = action_space
        .locate(base_space, frame_state.predicted_display_time)
        .unwrap();
    let pose_valid = location
        .location_flags
        .intersects(SpaceLocationFlags::POSITION_VALID | SpaceLocationFlags::ORIENTATION_VALID);
    if pose_valid {
        Some(super::transform(&location.pose))
    } else {
        None
    }
}

fn locate_hand<G: Graphics>(
    base_space: &Space,
    tracker: &HandTracker,
    frame_state: &FrameState,
    use_alternate_input_source: bool,
    session: &Session<G>,
    aim_state: &mut Option<HandTrackingAimStateFB>,
) -> Option<Box<Hand<JointFrame>>> {
    let mut state = HandTrackingAimStateFB::out(std::ptr::null_mut());
    let locations = {
        if !use_alternate_input_source {
            base_space.locate_hand_joints(tracker, frame_state.predicted_display_time)
        } else {
            let locate_info = HandJointsLocateInfoEXT {
                ty: HandJointsLocateInfoEXT::TYPE,
                next: std::ptr::null(),
                base_space: base_space.as_raw(),
                time: frame_state.predicted_display_time,
            };

            let mut locations = MaybeUninit::<[HandJointLocation; HAND_JOINT_COUNT]>::uninit();
            let mut location_info = HandJointLocationsEXT {
                ty: HandJointLocationsEXT::TYPE,
                next: &mut state as *mut _ as *mut c_void,
                is_active: false.into(),
                joint_count: HAND_JOINT_COUNT as u32,
                joint_locations: locations.as_mut_ptr() as _,
            };

            // Check if hand tracking is supported by the session instance
            let raw_hand_tracker = session.instance().exts().ext_hand_tracking.as_ref()?;

            unsafe {
                Ok(
                    match (raw_hand_tracker.locate_hand_joints)(
                        tracker.as_raw(),
                        &locate_info,
                        &mut location_info,
                    ) {
                        openxr::sys::Result::SUCCESS if location_info.is_active.into() => {
                            aim_state.replace(state.assume_init());
                            Some(locations.assume_init())
                        }
                        _ => None,
                    },
                )
            }
        }
    };
    let locations = if let Ok(Some(ref locations)) = locations {
        Hand {
            wrist: Some(&locations[HandJoint::WRIST]),
            thumb_metacarpal: Some(&locations[HandJoint::THUMB_METACARPAL]),
            thumb_phalanx_proximal: Some(&locations[HandJoint::THUMB_PROXIMAL]),
            thumb_phalanx_distal: Some(&locations[HandJoint::THUMB_DISTAL]),
            thumb_phalanx_tip: Some(&locations[HandJoint::THUMB_TIP]),
            index: Finger {
                metacarpal: Some(&locations[HandJoint::INDEX_METACARPAL]),
                phalanx_proximal: Some(&locations[HandJoint::INDEX_PROXIMAL]),
                phalanx_intermediate: Some(&locations[HandJoint::INDEX_INTERMEDIATE]),
                phalanx_distal: Some(&locations[HandJoint::INDEX_DISTAL]),
                phalanx_tip: Some(&locations[HandJoint::INDEX_TIP]),
            },
            middle: Finger {
                metacarpal: Some(&locations[HandJoint::MIDDLE_METACARPAL]),
                phalanx_proximal: Some(&locations[HandJoint::MIDDLE_PROXIMAL]),
                phalanx_intermediate: Some(&locations[HandJoint::MIDDLE_INTERMEDIATE]),
                phalanx_distal: Some(&locations[HandJoint::MIDDLE_DISTAL]),
                phalanx_tip: Some(&locations[HandJoint::MIDDLE_TIP]),
            },
            ring: Finger {
                metacarpal: Some(&locations[HandJoint::RING_METACARPAL]),
                phalanx_proximal: Some(&locations[HandJoint::RING_PROXIMAL]),
                phalanx_intermediate: Some(&locations[HandJoint::RING_INTERMEDIATE]),
                phalanx_distal: Some(&locations[HandJoint::RING_DISTAL]),
                phalanx_tip: Some(&locations[HandJoint::RING_TIP]),
            },
            little: Finger {
                metacarpal: Some(&locations[HandJoint::LITTLE_METACARPAL]),
                phalanx_proximal: Some(&locations[HandJoint::LITTLE_PROXIMAL]),
                phalanx_intermediate: Some(&locations[HandJoint::LITTLE_INTERMEDIATE]),
                phalanx_distal: Some(&locations[HandJoint::LITTLE_DISTAL]),
                phalanx_tip: Some(&locations[HandJoint::LITTLE_TIP]),
            },
        }
    } else {
        return None;
    };

    Some(Box::new(locations.map(|loc, _| {
        loc.and_then(|location| {
            let pose_valid = location.location_flags.intersects(
                SpaceLocationFlags::POSITION_VALID | SpaceLocationFlags::ORIENTATION_VALID,
            );
            if pose_valid {
                Some(JointFrame {
                    pose: super::transform(&location.pose),
                    radius: location.radius,
                })
            } else {
                None
            }
        })
    })))
}
