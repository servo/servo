/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Content process initialization, for multiprocess mode.

use platform::sandbox;

use compositing::content_process::{BootstrapInfo, ContentProcess, ContentProcessIpc};
use gfx::font_cache_task::FontCacheTask;
use gfx::paint_task::LayoutToPaintChan;
use layout::layout_task::LayoutTask;
use libc::c_int;
use script::script_task::ScriptTask;
use serialize::{Decodable, Encodable};
use servo_msg::constellation_msg::ConstellationChan;
use servo_net::image_cache_task::ImageCacheTask;
use servo_net::resource_task;
use servo_net::server::ServerProxy;
use servo_net::storage_task::StorageTask;
use servo_util::ipc::{IpcReceiver, IpcSender};
use servo_util::opts;
use servo_util::sbsf::{ServoDecoder, ServoEncoder};
use servo_util::taskpool::TaskPool;
use servo_util::time::TimeProfiler;
use std::io::IoError;
use std::sync::{Arc, Mutex};

pub fn main(bootstrap_fd: c_int) {
    let bootstrap_receiver = connect_ipc_receiver(bootstrap_fd);
    let bootstrap_info: BootstrapInfo = bootstrap_receiver.recv();

    // Must enter the sandbox *first* since on Linux seccomp only applies to the calling thread.
    sandbox::enter(bootstrap_info.auxiliary_data.zone.clone());

    opts::set_opts(bootstrap_info.opts);

    let shared_task_pool = TaskPool::new(8);
    let resource_task = resource_task::new_resource_task(None);
    let image_cache_task = ImageCacheTask::new(resource_task.clone(), shared_task_pool);
    let time_profiler_chan = TimeProfiler::create(None);
    let content_process: ContentProcess<LayoutTask,ScriptTask> = ContentProcess {
        ipc: ContentProcessIpc {
            script_to_compositor_client: Arc::new(Mutex::new(connect_ipc_server(
                                                     bootstrap_info.script_to_compositor_client))),
            script_port: connect_ipc_receiver(bootstrap_info.script_port),
            constellation_chan: ConstellationChan::from_server_proxy(Arc::new(Mutex::new(
                        connect_ipc_server(bootstrap_info.constellation_chan)))),
            storage_task: StorageTask::from_client(Arc::new(Mutex::new(
                        connect_ipc_server(bootstrap_info.storage_task)))),
            pipeline_to_layout_port: connect_ipc_receiver(bootstrap_info.pipeline_to_layout_port),
            layout_to_paint_chan: LayoutToPaintChan::from_channel(
                connect_ipc_sender(bootstrap_info.layout_to_paint_chan)),
            font_cache_task: FontCacheTask::from_client(Arc::new(Mutex::new(
                        connect_ipc_server(bootstrap_info.font_cache_task)))),
        },
        resource_task: resource_task,
        image_cache_task: image_cache_task,
        time_profiler_chan: time_profiler_chan,
    };

    content_process.create_script_and_layout_threads(bootstrap_info.auxiliary_data);
}

fn connect_ipc_receiver<T>(fd: c_int)
                           -> IpcReceiver<T>
                           where T: for<'a> Decodable<ServoDecoder<'a>,IoError> {
    IpcReceiver::from_fd(fd)
}

fn connect_ipc_sender<T>(fd: c_int)
                         -> IpcSender<T>
                         where T: for<'a> Encodable<ServoEncoder<'a>,IoError> {
    IpcSender::from_fd(fd)
}

fn connect_ipc_server<M,R>((sender_fd, receiver_fd): (c_int, c_int)) -> ServerProxy<M,R>
                           where M: for<'a> Decodable<ServoDecoder<'a>,IoError> +
                                    for<'a> Encodable<ServoEncoder<'a>,IoError>,
                                 R: for<'a> Decodable<ServoDecoder<'a>,IoError> +
                                    for<'a> Encodable<ServoEncoder<'a>,IoError> {
    ServerProxy::from_fds(sender_fd, receiver_fd)
}

