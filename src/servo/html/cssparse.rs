/*!
Some little helpers for hooking up the HTML parser with the CSS parser
*/

use std::net::url::Url;
use resource::resource_task::{ResourceTask, ProgressMsg, Load, Payload, Done};
use newcss::values::Rule;
use newcss::lexer_util::DataStream;
use newcss::lexer::{Token, lex_css_from_bytes};

pub fn spawn_css_parser(url: Url, resource_task: ResourceTask) -> comm::Port<~[~Rule]> {
    let result_port = comm::Port();
    let result_chan = comm::Chan(&result_port);
    // TODO: change copy to move once we have match move
    let url = copy url;
    do task::spawn |move url, copy resource_task| {
        let css_stream = spawn_css_lexer_task(copy url, resource_task);
        let mut css_rules = newcss::parser::build_stylesheet(move css_stream);
        result_chan.send(move css_rules);
    }

    return result_port;
}

#[allow(non_implicitly_copyable_typarams)]
fn spawn_css_lexer_task(url: Url, resource_task: ResourceTask) -> pipes::Port<Token> {
    let (result_chan, result_port) = pipes::stream();

    do task::spawn |move result_chan, move url| {
        assert url.path.ends_with(".css");
        let input_port = Port();
        // TODO: change copy to move once the compiler permits it
        resource_task.send(Load(copy url, input_port.chan()));

        lex_css_from_bytes(resource_port_to_lexer_stream(input_port), &result_chan);
    };

    return move result_port;
}

fn resource_port_to_lexer_stream(input_port: comm::Port<ProgressMsg>) -> DataStream {
    return || {
        match input_port.recv() {
            Payload(move data) => Some(move data),
            Done(*) => None
        }
    }
}
