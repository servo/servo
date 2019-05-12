/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::dom::audionode::AudioNode;
use crate::dom::baseaudiocontext::BaseAudioContext;
use crate::dom::bindings::cell::DomRefCell;
use crate::dom::bindings::codegen::Bindings::AnalyserNodeBinding::{
    self, AnalyserNodeMethods, AnalyserOptions,
};
use crate::dom::bindings::codegen::Bindings::AudioNodeBinding::{
    ChannelCountMode, ChannelInterpretation,
};
use crate::dom::bindings::error::{Error, Fallible};
use crate::dom::bindings::num::Finite;
use crate::dom::bindings::refcounted::Trusted;
use crate::dom::bindings::reflector::reflect_dom_object;
use crate::dom::bindings::root::DomRoot;
use crate::dom::window::Window;
use crate::task_source::TaskSource;
use dom_struct::dom_struct;
use ipc_channel::ipc::{self, IpcReceiver};
use ipc_channel::router::ROUTER;
use js::rust::CustomAutoRooterGuard;
use js::typedarray::{Float32Array, Uint8Array};
use servo_media::audio::analyser_node::AnalysisEngine;
use servo_media::audio::block::Block;
use servo_media::audio::node::AudioNodeInit;

#[dom_struct]
pub struct AnalyserNode {
    node: AudioNode,
    #[ignore_malloc_size_of = "Defined in servo-media"]
    engine: DomRefCell<AnalysisEngine>,
}

impl AnalyserNode {
    #[allow(unrooted_must_root)]
    pub fn new_inherited(
        _: &Window,
        context: &BaseAudioContext,
        options: &AnalyserOptions,
    ) -> Fallible<(AnalyserNode, IpcReceiver<Block>)> {
        let node_options =
            options
                .parent
                .unwrap_or(2, ChannelCountMode::Max, ChannelInterpretation::Speakers);

        if options.fftSize > 32768 ||
            options.fftSize < 32 ||
            (options.fftSize & (options.fftSize - 1) != 0)
        {
            return Err(Error::IndexSize);
        }

        if *options.maxDecibels <= *options.minDecibels {
            return Err(Error::IndexSize);
        }

        if *options.smoothingTimeConstant < 0. || *options.smoothingTimeConstant > 1. {
            return Err(Error::IndexSize);
        }

        let (send, rcv) = ipc::channel().unwrap();
        let callback = move |block| {
            send.send(block).unwrap();
        };

        let node = AudioNode::new_inherited(
            AudioNodeInit::AnalyserNode(Box::new(callback)),
            context,
            node_options,
            1, // inputs
            1, // outputs
        )?;

        let engine = AnalysisEngine::new(
            options.fftSize as usize,
            *options.smoothingTimeConstant,
            *options.minDecibels,
            *options.maxDecibels,
        );
        Ok((
            AnalyserNode {
                node,
                engine: DomRefCell::new(engine),
            },
            rcv,
        ))
    }

    #[allow(unrooted_must_root)]
    pub fn new(
        window: &Window,
        context: &BaseAudioContext,
        options: &AnalyserOptions,
    ) -> Fallible<DomRoot<AnalyserNode>> {
        let (node, recv) = AnalyserNode::new_inherited(window, context, options)?;
        let object = reflect_dom_object(Box::new(node), window, AnalyserNodeBinding::Wrap);
        let (source, canceller) = window
            .task_manager()
            .media_element_task_source_with_canceller();
        let this = Trusted::new(&*object);

        ROUTER.add_route(
            recv.to_opaque(),
            Box::new(move |block| {
                let this = this.clone();
                let _ = source.queue_with_canceller(
                    task!(append_analysis_block: move || {
                        let this = this.root();
                        this.push_block(block.to().unwrap())
                    }),
                    &canceller,
                );
            }),
        );
        Ok(object)
    }

    /// https://webaudio.github.io/web-audio-api/#dom-analysernode-analysernode
    pub fn Constructor(
        window: &Window,
        context: &BaseAudioContext,
        options: &AnalyserOptions,
    ) -> Fallible<DomRoot<AnalyserNode>> {
        AnalyserNode::new(window, context, options)
    }

    pub fn push_block(&self, block: Block) {
        self.engine.borrow_mut().push(block)
    }
}

impl AnalyserNodeMethods for AnalyserNode {
    #[allow(unsafe_code)]
    /// https://webaudio.github.io/web-audio-api/#dom-analysernode-getfloatfrequencydata
    fn GetFloatFrequencyData(&self, mut array: CustomAutoRooterGuard<Float32Array>) {
        // Invariant to maintain: No JS code that may touch the array should
        // run whilst we're writing to it
        let dest = unsafe { array.as_mut_slice() };
        self.engine.borrow_mut().fill_frequency_data(dest);
    }

    #[allow(unsafe_code)]
    /// https://webaudio.github.io/web-audio-api/#dom-analysernode-getbytefrequencydata
    fn GetByteFrequencyData(&self, mut array: CustomAutoRooterGuard<Uint8Array>) {
        // Invariant to maintain: No JS code that may touch the array should
        // run whilst we're writing to it
        let dest = unsafe { array.as_mut_slice() };
        self.engine.borrow_mut().fill_byte_frequency_data(dest);
    }

    #[allow(unsafe_code)]
    /// https://webaudio.github.io/web-audio-api/#dom-analysernode-getfloattimedomaindata
    fn GetFloatTimeDomainData(&self, mut array: CustomAutoRooterGuard<Float32Array>) {
        // Invariant to maintain: No JS code that may touch the array should
        // run whilst we're writing to it
        let dest = unsafe { array.as_mut_slice() };
        self.engine.borrow().fill_time_domain_data(dest);
    }

    #[allow(unsafe_code)]
    /// https://webaudio.github.io/web-audio-api/#dom-analysernode-getbytetimedomaindata
    fn GetByteTimeDomainData(&self, mut array: CustomAutoRooterGuard<Uint8Array>) {
        // Invariant to maintain: No JS code that may touch the array should
        // run whilst we're writing to it
        let dest = unsafe { array.as_mut_slice() };
        self.engine.borrow().fill_byte_time_domain_data(dest);
    }

    /// https://webaudio.github.io/web-audio-api/#dom-analysernode-fftsize
    fn SetFftSize(&self, value: u32) -> Fallible<()> {
        if value > 32768 || value < 32 || (value & (value - 1) != 0) {
            return Err(Error::IndexSize);
        }
        self.engine.borrow_mut().set_fft_size(value as usize);
        Ok(())
    }

    /// https://webaudio.github.io/web-audio-api/#dom-analysernode-fftsize
    fn FftSize(&self) -> u32 {
        self.engine.borrow().get_fft_size() as u32
    }

    /// https://webaudio.github.io/web-audio-api/#dom-analysernode-frequencybincount
    fn FrequencyBinCount(&self) -> u32 {
        self.FftSize() / 2
    }

    /// https://webaudio.github.io/web-audio-api/#dom-analysernode-mindecibels
    fn MinDecibels(&self) -> Finite<f64> {
        Finite::wrap(self.engine.borrow().get_min_decibels())
    }

    /// https://webaudio.github.io/web-audio-api/#dom-analysernode-mindecibels
    fn SetMinDecibels(&self, value: Finite<f64>) -> Fallible<()> {
        if *value >= self.engine.borrow().get_max_decibels() {
            return Err(Error::IndexSize);
        }
        self.engine.borrow_mut().set_min_decibels(*value);
        Ok(())
    }

    /// https://webaudio.github.io/web-audio-api/#dom-analysernode-maxdecibels
    fn MaxDecibels(&self) -> Finite<f64> {
        Finite::wrap(self.engine.borrow().get_max_decibels())
    }

    /// https://webaudio.github.io/web-audio-api/#dom-analysernode-maxdecibels
    fn SetMaxDecibels(&self, value: Finite<f64>) -> Fallible<()> {
        if *value <= self.engine.borrow().get_min_decibels() {
            return Err(Error::IndexSize);
        }
        self.engine.borrow_mut().set_max_decibels(*value);
        Ok(())
    }

    /// https://webaudio.github.io/web-audio-api/#dom-analysernode-smoothingtimeconstant
    fn SmoothingTimeConstant(&self) -> Finite<f64> {
        Finite::wrap(self.engine.borrow().get_smoothing_constant())
    }

    /// https://webaudio.github.io/web-audio-api/#dom-analysernode-smoothingtimeconstant
    fn SetSmoothingTimeConstant(&self, value: Finite<f64>) -> Fallible<()> {
        if *value < 0. || *value > 1. {
            return Err(Error::IndexSize);
        }
        self.engine.borrow_mut().set_smoothing_constant(*value);
        Ok(())
    }
}
