/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use gfx::font_cache_task::FontCacheTask;
use gfx::paint_task::LayoutToPaintChan;
use layout_traits::{LayoutControlMsg, LayoutTaskFactory};
use libc::c_int;
use libc::funcs::posix88::unistd;
use script_traits::{ConstellationControlMsg, ScriptTaskFactory};
use servo_msg::compositor_msg::ScriptToCompositorMsg;
use servo_msg::constellation_msg::{ConstellationChan, Failure, LoadData, PipelineId};
use servo_msg::constellation_msg::{WindowSizeData};
use servo_net::image_cache_task::ImageCacheTask;
use servo_net::resource_task::ResourceTask;
use servo_net::server::SharedServerProxy;
use servo_net::storage_task::StorageTask;
use servo_util::ipc::{mod, IpcReceiver};
use servo_util::opts::{mod, Opts};
use servo_util::time::TimeProfilerChan;
use std::os;
use std::ptr;

/// Messages from the chrome process to the content process.
#[deriving(Encodable, Decodable)]
pub enum ContentProcessMsg {
    CreateScriptAndLayoutThreads(AuxiliaryContentProcessData),
}

/// IPC channels that a content process uses to communicate with the chrome process.
pub struct ContentProcessIpc {
    pub script_to_compositor_client: SharedServerProxy<ScriptToCompositorMsg,()>,
    pub script_port: IpcReceiver<ConstellationControlMsg>,
    pub constellation_chan: ConstellationChan,
    pub storage_task: StorageTask,
    pub pipeline_to_layout_port: IpcReceiver<LayoutControlMsg>,
    pub layout_to_paint_chan: LayoutToPaintChan,
    pub font_cache_task: FontCacheTask,
}

/// Other data used to construct a content process.
#[deriving(Encodable, Decodable)]
pub struct AuxiliaryContentProcessData {
    pub pipeline_id: PipelineId,
    pub failure: Failure,
    pub window_size: WindowSizeData,
    pub zone: Zone,
}

/// Information sent over IPC to bootstrap a content process. This consists of the auxiliary
/// content process data plus a set of file descriptors that can construct a `ContentProcessIpc`.
#[deriving(Encodable, Decodable)]
pub struct BootstrapInfo {
    pub script_to_compositor_client: (c_int, c_int),
    pub script_port: c_int,
    pub constellation_chan: (c_int, c_int),
    pub storage_task: (c_int, c_int),
    pub pipeline_to_layout_port: c_int,
    pub layout_to_paint_chan: c_int,
    pub font_cache_task: (c_int, c_int),
    pub auxiliary_data: AuxiliaryContentProcessData,
    pub opts: Opts,
}

pub struct ContentProcess<LTF,STF> where LTF: LayoutTaskFactory, STF: ScriptTaskFactory {
    pub ipc: ContentProcessIpc,
    pub resource_task: ResourceTask,
    pub image_cache_task: ImageCacheTask,
    pub time_profiler_chan: TimeProfilerChan,
}

impl<LTF,STF> ContentProcess<LTF,STF> where LTF: LayoutTaskFactory, STF: ScriptTaskFactory {
    pub fn create_script_and_layout_threads(self, data: AuxiliaryContentProcessData) {
        let layout_pair = ScriptTaskFactory::create_layout_channel(None::<&mut STF>);
        let (layout_to_script_sender, layout_to_script_receiver) = channel();
        ScriptTaskFactory::create(None::<&mut STF>,
                                  data.pipeline_id,
                                  self.ipc.script_to_compositor_client,
                                  &layout_pair,
                                  self.ipc.script_port,
                                  self.ipc.constellation_chan.clone(),
                                  data.failure.clone(),
                                  layout_to_script_receiver,
                                  self.resource_task.clone(),
                                  self.ipc.storage_task,
                                  self.image_cache_task.clone(),
                                  None,
                                  data.window_size);
        LayoutTaskFactory::create(None::<&mut LTF>,
                                  data.pipeline_id,
                                  layout_pair,
                                  self.ipc.pipeline_to_layout_port,
                                  self.ipc.constellation_chan,
                                  data.failure,
                                  layout_to_script_sender,
                                  self.ipc.layout_to_paint_chan,
                                  self.resource_task.clone(),
                                  self.image_cache_task.clone(),
                                  self.ipc.font_cache_task,
                                  self.time_profiler_chan);
    }
}

/// Spawns a content process.
///
/// FIXME(pcwalton): This leaks most of the file descriptors. :( We will need to wait for Rust to
/// use `O_CLOEXEC` properly.
pub fn spawn(ipc: ContentProcessIpc, data: AuxiliaryContentProcessData) {
    let (bootstrap_receiver, bootstrap_sender) = ipc::channel();
    let bootstrap_info = BootstrapInfo {
        script_to_compositor_client: ipc.script_to_compositor_client.lock().fds(),
        script_port: ipc.script_port.fd(),
        constellation_chan: ipc.constellation_chan.server_proxy().lock().fds(),
        storage_task: ipc.storage_task.client.lock().fds(),
        pipeline_to_layout_port: ipc.pipeline_to_layout_port.fd(),
        layout_to_paint_chan: ipc.layout_to_paint_chan.fd(),
        font_cache_task: ipc.font_cache_task.client.lock().fds(),
        auxiliary_data: data,
        opts: (*opts::get()).clone(),
    };

    let args = [os::self_exe_name().expect("can't find our own executable name")];
    let argv: Vec<_> = args.iter().map(|arg| arg.to_c_str()).collect();
    let mut argv: Vec<_> = argv.iter().map(|arg| arg.as_ptr()).collect();
    argv.push(ptr::null());

    os::setenv("SERVO_CONTENT_PROCESS", bootstrap_receiver.fd().to_string());

    unsafe {
        if unistd::fork() == 0 {
            unistd::execv(argv[0], &mut argv[0]);
            panic!("failed to execute content process: {}!", args[0].display());
        }
    }

    ipc.storage_task.client.lock().forget();
    ipc.font_cache_task.client.lock().forget();

    bootstrap_sender.send(bootstrap_info);
}

/// The zone that the content process is restricted to. At present, this determines whether local
/// files can be accessed.
///
/// TODO(pcwalton): Remove this once the resource task has been rewritten to be e10s-safe. At that
/// point the resource task can handle this policy and the sandbox will not have to be involved.
#[deriving(Clone, Encodable, Decodable)]
pub enum Zone {
    /// The local zone. This allows full access to both local files and remote resources.
    Local,
    /// The remote zone. This allows access to remote resources but disallows access to local
    /// files.
    Remote,
}

impl Zone {
    pub fn from_load_data(load_data: &LoadData) -> Zone {
        if load_data.url.scheme.as_slice() == "file" {
            Zone::Local
        } else {
            Zone::Remote
        }
    }
}

