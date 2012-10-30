/*!
Some little helpers for hooking up the HTML parser with the CSS parser
*/

use std::net::url::Url;
use std::cell::Cell;
use resource::resource_task::{ResourceTask, ProgressMsg, Load, Payload, Done};
use newcss::values::Stylesheet;
use newcss::util::{DataStream, DataStreamFactory};

pub fn spawn_css_parser(url: Url, resource_task: ResourceTask) -> comm::Port<Stylesheet> {
    let result_port = comm::Port();
    let result_chan = comm::Chan(&result_port);
    do task::spawn |move url, copy resource_task| {
        let sheet = newcss::parser::parse_stylesheet(copy url, data_stream_factory(copy url, resource_task));
        result_chan.send(move sheet);
    }

    return result_port;
}

fn data_stream_factory(url: Url, resource_task: ResourceTask) -> DataStreamFactory {
    let url = Cell(move url);
    return |move url| {
        let input_port = Port();
        resource_task.send(Load(url.take(), input_port.chan()));
        resource_port_to_data_stream(input_port)
    }
}

fn resource_port_to_data_stream(input_port: comm::Port<ProgressMsg>) -> DataStream {
    return || {
        match input_port.recv() {
            Payload(move data) => Some(move data),
            Done(*) => None
        }
    }
}
