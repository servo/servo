/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

#![allow(unrooted_must_root)]

use html5ever::buffer_queue::BufferQueue;
use html5ever::tokenizer::Token;
use html5ever::tokenizer::TokenSink;
use html5ever::tokenizer::TokenSinkResult;
use html5ever::tokenizer::Tokenizer as HtmlTokenizer;
use net_traits::request::RequestBuilder;
use net_traits::CoreResourceMsg;
use net_traits::FetchChannels;
use net_traits::IpcSend;
use net_traits::ResourceThreads;
use servo_url::ServoUrl;

#[derive(MallocSizeOf)]
#[must_root]
pub struct Tokenizer {
    #[ignore_malloc_size_of = "Defined in html5ever"]
    inner: HtmlTokenizer<PrefetchSink>,
}

unsafe_no_jsmanaged_fields!(Tokenizer);

impl Tokenizer {
    pub fn new(base: ServoUrl, resource_threads: ResourceThreads) -> Self {
        let sink = PrefetchSink {
            base,
            resource_threads,
        };
        let options = Default::default();
        let inner = HtmlTokenizer::new(sink, options);
        Tokenizer { inner }
    }

    pub fn feed(&mut self, input: &mut BufferQueue) {
        let _ = self.inner.feed(input);
    }
}

struct PrefetchSink {
    base: ServoUrl,
    resource_threads: ResourceThreads,
}

impl TokenSink for PrefetchSink {
    type Handle = ();
    fn process_token(&mut self, token: Token, _line_number: u64) -> TokenSinkResult<()> {
        if let Token::TagToken(ref tag) = token {
            if (tag.name == local_name!("img")) || (tag.name == local_name!("script")) {
                let srcs = tag
                    .attrs
                    .iter()
                    .filter(|attr| attr.name.local == local_name!("src"));
                for src in srcs {
                    if let Ok(url) = ServoUrl::parse_with_base(Some(&self.base), &src.value) {
                        debug!("Prefetch {} {}", tag.name, url);
                        let req_init = RequestBuilder::new(url);
                        let _ = self
                            .resource_threads
                            .send(CoreResourceMsg::Fetch(req_init, FetchChannels::Prefetch));
                    }
                }
            }
        }
        TokenSinkResult::Continue
    }
}
