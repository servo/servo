/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

#![recursion_limit = "128"]

extern crate proc_macro;
extern crate proc_macro2;
extern crate syn;
#[macro_use]
extern crate quote;

use proc_macro::TokenStream;

#[proc_macro_derive(AudioScheduledSourceNode)]
pub fn audio_scheduled_source_node(input: TokenStream) -> TokenStream {
    let ast: syn::DeriveInput = syn::parse(input).unwrap();
    let r#gen = impl_audio_scheduled_source_node(&ast);
    r#gen.into()
}

fn impl_audio_scheduled_source_node(ast: &syn::DeriveInput) -> proc_macro2::TokenStream {
    let name = &ast.ident;
    quote! {
        impl #name {
            fn should_play_at(&mut self, tick: Tick) -> ShouldPlay {
                let start = if let Some(start) = self.start_at {
                    start
                } else {
                    return ShouldPlay::No;
                };

                let frame_end = tick + Tick::FRAMES_PER_BLOCK;
                if tick < start {
                    if frame_end < start {
                        ShouldPlay::No
                    } else {
                        let delta_start = start - tick;
                        if let Some(stop) = self.stop_at {
                            if stop <= start {
                                self.maybe_trigger_onended_callback();
                                return ShouldPlay::No;
                            }
                            if stop > frame_end {
                                ShouldPlay::Between(delta_start, Tick::FRAMES_PER_BLOCK)
                            } else {
                                self.maybe_trigger_onended_callback();
                                ShouldPlay::Between(delta_start, stop - tick)
                            }
                        } else {
                            ShouldPlay::Between(delta_start, Tick::FRAMES_PER_BLOCK)
                        }
                    }
                } else {
                    let stop = if let Some(stop) = self.stop_at {
                        stop
                    } else {
                        return ShouldPlay::Between(Tick(0), Tick::FRAMES_PER_BLOCK);
                    };
                    if stop > frame_end {
                        ShouldPlay::Between(Tick(0), Tick::FRAMES_PER_BLOCK)
                    } else if stop < tick {
                        self.maybe_trigger_onended_callback();
                        ShouldPlay::No
                    } else {
                        self.maybe_trigger_onended_callback();
                        ShouldPlay::Between(Tick(0), stop - tick)
                    }
                }
            }

            fn start(&mut self, tick: Tick) -> bool {
                // We can only allow a single call to `start` and always before
                // any `stop` calls.
                if self.start_at.is_some() || self.stop_at.is_some() {
                    return false;
                }
                self.start_at = Some(tick);
                true
            }

            fn stop(&mut self, tick: Tick) -> bool {
                // We can only allow calls to `stop` after `start` is called.
                if self.start_at.is_none() {
                    return false;
                }
                // If `stop` is called again after already having been called,
                // the last invocation will be the only one applied.
                self.stop_at = Some(tick);
                true
            }

            fn maybe_trigger_onended_callback(&mut self) {
                // We cannot have an end without a start.
                if self.start_at.is_none() {
                    return;
                }
                if let Some(cb) = self.onended_callback.take() {
                    cb.0()
                }
            }

            fn handle_source_node_message(&mut self, message: AudioScheduledSourceNodeMessage, sample_rate: f32) {
                match message {
                    AudioScheduledSourceNodeMessage::Start(when) => {
                        self.start(Tick::from_time(when, sample_rate));
                    }
                    AudioScheduledSourceNodeMessage::Stop(when) => {
                        self.stop(Tick::from_time(when, sample_rate));
                    }
                    AudioScheduledSourceNodeMessage::RegisterOnEndedCallback(callback) => {
                        self.onended_callback = Some(callback);
                    }
                }
            }
        }
    }
}

#[proc_macro_derive(AudioNodeCommon)]
pub fn channel_info(input: TokenStream) -> TokenStream {
    let ast: syn::DeriveInput = syn::parse(input).unwrap();
    let name = &ast.ident;
    let r#gen = quote! {
        impl crate::node::AudioNodeCommon for #name {
            fn channel_info(&self) -> &crate::node::ChannelInfo {
                &self.channel_info
            }

            fn channel_info_mut(&mut self) -> &mut crate::node::ChannelInfo {
                &mut self.channel_info
            }
        }
    };
    r#gen.into()
}
