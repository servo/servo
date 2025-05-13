/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::collections::HashMap;

use base::id::WebViewId;
use constellation_traits::EmbedderToConstellationMessage;
use embedder_traits::{JSValue, JavaScriptEvaluationError, JavaScriptEvaluationId};

use crate::ConstellationProxy;

struct PendingEvaluation {
    callback: Box<dyn FnOnce(Result<JSValue, JavaScriptEvaluationError>)>,
}

pub(crate) struct JavaScriptEvaluator {
    current_id: JavaScriptEvaluationId,
    constellation_proxy: ConstellationProxy,
    pending_evaluations: HashMap<JavaScriptEvaluationId, PendingEvaluation>,
}

impl JavaScriptEvaluator {
    pub(crate) fn new(constellation_proxy: ConstellationProxy) -> Self {
        Self {
            current_id: JavaScriptEvaluationId(0),
            constellation_proxy,
            pending_evaluations: Default::default(),
        }
    }

    fn generate_id(&mut self) -> JavaScriptEvaluationId {
        let next_id = JavaScriptEvaluationId(self.current_id.0 + 1);
        std::mem::replace(&mut self.current_id, next_id)
    }

    pub(crate) fn evaluate(
        &mut self,
        webview_id: WebViewId,
        script: String,
        callback: Box<dyn FnOnce(Result<JSValue, JavaScriptEvaluationError>)>,
    ) {
        let evaluation_id = self.generate_id();
        self.constellation_proxy
            .send(EmbedderToConstellationMessage::EvaluateJavaScript(
                webview_id,
                evaluation_id,
                script,
            ));
        self.pending_evaluations
            .insert(evaluation_id, PendingEvaluation { callback });
    }

    pub(crate) fn finish_evaluation(
        &mut self,
        evaluation_id: JavaScriptEvaluationId,
        result: Result<JSValue, JavaScriptEvaluationError>,
    ) {
        (self
            .pending_evaluations
            .remove(&evaluation_id)
            .expect("Received request to finish unknown JavaScript evaluation.")
            .callback)(result)
    }
}
