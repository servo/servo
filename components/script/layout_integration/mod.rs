use std::borrow::Cow;
use msg::constellation_msg::{BackgroundHangMonitorRegister, PipelineId};
use msg::constellation_msg::TopLevelBrowsingContextId;
use servo_url::ServoUrl;
use crossbeam_channel::{Receiver, Sender};
use ipc_channel::ipc::{IpcReceiver, IpcSender};
use net_traits::image_cache::ImageCache;
use script_traits::LayoutMsg;
use script_traits::{
    ConstellationControlMsg, LayoutControlMsg, WindowSizeData, WebrenderIpcSender,
};
use std::sync::atomic::AtomicBool;
use std::sync::Arc;

pub mod reflow;

pub trait ScriptThreadFactory {
    /// Type of message sent from script to layout.
    type Message;

    /// Create a `ScriptThread`.
    fn create<LTF: LayoutThreadFactory<Message = Self::Message>>(
        state: script_traits::InitialScriptState,
        load_data: script_traits::LoadData,
        profile_script_events: bool,
        print_pwm: bool,
        relayout_event: bool,
        prepare_for_screenshot: bool,
        unminify_js: bool,
        local_script_source: Option<String>,
        userscripts_path: Option<String>,
        headless: bool,
        replace_surrogates: bool,
        user_agent: Cow<'static, str>,
        layout_init: script_traits::LayoutInit,
    ) /*-> (Sender<Self::Message>, Receiver<Self::Message>)*/;
}

pub trait Layout {
    fn process(&mut self, msg: script_layout_interface::message::Msg);
    fn rpc(&self) -> Box<dyn script_layout_interface::rpc::LayoutRPC>;
    fn reflow<'a>(&mut self, reflow: reflow::ScriptReflow<'a>) -> reflow::ReflowComplete<'a>;
    fn handle_constellation_msg(&mut self, msg: script_traits::LayoutControlMsg);
    fn handle_font_cache_msg(&mut self);
    fn create_new_layout(&self, init: script_layout_interface::message::LayoutThreadInit) -> Box<dyn Layout>;
}

pub trait LayoutThreadFactory {
    type Message;
    fn create(
        id: PipelineId,
        top_level_browsing_context_id: TopLevelBrowsingContextId,
        url: ServoUrl,
        is_iframe: bool,
        chan: (Sender<Self::Message>, Receiver<Self::Message>),
        pipeline_port: IpcReceiver<LayoutControlMsg>,
        background_hang_monitor: Box<dyn BackgroundHangMonitorRegister>,
        constellation_chan: IpcSender<LayoutMsg>,
        script_chan: IpcSender<ConstellationControlMsg>,
        image_cache: Arc<dyn ImageCache>,
        font_cache_thread: gfx::font_cache_thread::FontCacheThread,
        time_profiler_chan: profile_traits::time::ProfilerChan,
        mem_profiler_chan: profile_traits::mem::ProfilerChan,
        webrender_api_sender: WebrenderIpcSender,
        //paint_time_metrics: metrics::PaintTimeMetrics,
        busy: Arc<AtomicBool>,
        load_webfonts_synchronously: bool,
        window_size: WindowSizeData,
        dump_display_list: bool,
        dump_display_list_json: bool,
        dump_style_tree: bool,
        dump_rule_tree: bool,
        relayout_event: bool,
        nonincremental_layout: bool,
        trace_layout: bool,
        dump_flow_tree: bool,
    ) -> Box<dyn Layout>;
}

