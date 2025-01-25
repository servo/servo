/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cell::RefCell;
use std::rc::Rc;

use base::id::WebViewId;
use compositing::IOCompositor;
use compositing_traits::ConstellationMsg;
use webrender_api::units::DeviceRect;

use crate::ConstellationProxy;

pub struct WebView(Rc<WebViewInner>);

struct WebViewInner {
    // TODO: ensure that WebView instances interact with the correct Servo instance
    pub(crate) id: WebViewId,
    pub(crate) constellation_proxy: ConstellationProxy,
    pub(crate) compositor: Rc<RefCell<IOCompositor>>,
}

impl Drop for WebViewInner {
    fn drop(&mut self) {
        self.constellation_proxy
            .send(ConstellationMsg::CloseWebView(self.id));
    }
}

/// Handle for a webview.
///
/// - The webview exists for exactly as long as there are WebView handles
///   (FIXME: this is not true yet; webviews can still close of their own volition)
/// - All methods are infallible; if the constellation dies, the embedder finds out when calling
///   [Servo::handle_events](crate::Servo::handle_events)
impl WebView {
    pub(crate) fn new(
        constellation_proxy: &ConstellationProxy,
        compositor: Rc<RefCell<IOCompositor>>,
        url: url::Url,
    ) -> Self {
        let webview_id = WebViewId::new();
        constellation_proxy.send(ConstellationMsg::NewWebView(url.into(), webview_id));

        Self(Rc::new(WebViewInner {
            id: webview_id,
            constellation_proxy: constellation_proxy.clone(),
            compositor,
        }))
    }

    /// FIXME: Remove this once we have a webview delegate.
    pub(crate) fn new_auxiliary(
        constellation_proxy: &ConstellationProxy,
        compositor: Rc<RefCell<IOCompositor>>,
    ) -> Self {
        let webview_id = WebViewId::new();

        Self(
            WebViewInner {
                id: webview_id,
                constellation_proxy: constellation_proxy.clone(),
                compositor,
            }
            .into(),
        )
    }

    /// FIXME: Remove this once we have a webview delegate.
    pub fn id(&self) -> WebViewId {
        self.0.id
    }

    pub fn focus(&self) {
        self.0
            .constellation_proxy
            .send(ConstellationMsg::FocusWebView(self.id()));
    }

    pub fn blur(&self) {
        self.0
            .constellation_proxy
            .send(ConstellationMsg::BlurWebView);
    }

    pub fn move_resize(&self, rect: DeviceRect) {
        self.0
            .compositor
            .borrow_mut()
            .move_resize_webview(self.id(), rect);
    }

    pub fn show(&self, hide_others: bool) {
        self.0
            .compositor
            .borrow_mut()
            .show_webview(self.id(), hide_others)
            .expect("BUG: invalid WebView instance");
    }

    pub fn hide(&self) {
        self.0
            .compositor
            .borrow_mut()
            .hide_webview(self.id())
            .expect("BUG: invalid WebView instance");
    }

    pub fn raise_to_top(&self, hide_others: bool) {
        self.0
            .compositor
            .borrow_mut()
            .raise_webview_to_top(self.id(), hide_others)
            .expect("BUG: invalid WebView instance");
    }
}
