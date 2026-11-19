/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

mod common;

use std::rc::Rc;
use std::sync::mpsc::{self, RecvTimeoutError};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use common::{ServoTest, WebViewDelegateImpl, evaluate_javascript};
use servo::WebViewBuilder;

#[test]
fn shared_worker_shutdown_interrupts_running_javascript() -> Result<(), String> {
    let (cancel, timeout) = mpsc::channel();
    let phase = Arc::new(Mutex::new("initializing Servo"));
    let watchdog_phase = phase.clone();
    let watchdog = std::thread::spawn(move || {
        if matches!(
            timeout.recv_timeout(Duration::from_secs(45)),
            Err(RecvTimeoutError::Timeout)
        ) {
            eprintln!(
                "SharedWorker test timed out while {}",
                watchdog_phase.lock().expect("watchdog phase"),
            );
            std::process::exit(1);
        }
    });
    let servo_test = ServoTest::new();
    let delegate = Rc::new(WebViewDelegateImpl::default());
    let webview = WebViewBuilder::new(servo_test.servo(), servo_test.rendering_context.clone())
        .delegate(delegate.clone())
        .build();
    *phase.lock().expect("watchdog phase") = "evaluating the constructor";
    evaluate_javascript(
        &servo_test,
        webview.clone(),
        r#"
        window.worker = new SharedWorker(URL.createObjectURL(new Blob([`
            onconnect = event => {
                const port = event.ports[0];
                port.onmessage = () => {
                    console.log("shared-worker-running");
                    while (true) {}
                };
                port.postMessage("connected");
            };
        `], {type: "text/javascript"})));
        worker.onerror = () => console.log("shared-worker-startup-error");
        worker.port.onmessage = () => worker.port.postMessage("run");
        undefined;
    "#,
    )
    .map_err(|error| format!("Could not construct SharedWorker: {error:?}"))?;

    *phase.lock().expect("watchdog phase") = "waiting for the running worker";
    let deadline = Instant::now() + Duration::from_secs(15);
    servo_test.spin(|| {
        Instant::now() < deadline &&
            !delegate
                .console_messages
                .borrow()
                .iter()
                .any(|(_, message)| {
                    message == "shared-worker-running" || message == "shared-worker-startup-error"
                })
    });
    if !delegate
        .console_messages
        .borrow()
        .iter()
        .any(|(_, message)| message == "shared-worker-running")
    {
        return Err(format!(
            "SharedWorker did not start: {:?}",
            delegate.console_messages.borrow()
        ));
    }

    *phase.lock().expect("watchdog phase") = "dropping the owner";
    drop(webview);
    *phase.lock().expect("watchdog phase") = "shutting down Servo";
    drop(servo_test);
    cancel
        .send(())
        .map_err(|error| format!("Could not cancel watchdog: {error}"))?;
    watchdog
        .join()
        .map_err(|_| "SharedWorker shutdown watchdog panicked".to_owned())?;
    Ok(())
}
