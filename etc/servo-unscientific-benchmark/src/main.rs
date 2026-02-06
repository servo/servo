/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */
use std::{
    collections::VecDeque,
    io::stdin,
    path::PathBuf,
    process::{Command, Stdio},
};
mod webdriver;
mod webdriver_tests;

use anyhow::{Context, Result, anyhow};
use clap::Parser;
use env_logger::Env;
use log::{error, info, warn};
use termion::{event::Key, input::TermRead};
use thirtyfour::WebDriver;
use which::which;

use crate::webdriver::sleep;
use crate::webdriver_tests::*;

/// Benchmark flags
#[derive(Clone, Debug, clap::Args)]
#[group(required = true, multiple = false)]
struct Benchmark {
    /// Start the heaptrack analysis.
    #[arg(short, long)]
    heaptrack: bool,

    #[arg(short('r'), long)]
    /// Start the perf record analysis.
    perf: bool,

    #[arg(short, long)]
    /// Special to only run servo for debugging the webdriver scripts.
    debug: bool,
}

/// A unscientific benchmark for servo
#[derive(Parser, Debug)]
#[command(version, about, disable_help_flag = true)]
struct Args {
    #[arg(short, long)]
    /// Use this if you are behind a proxy.
    proxychains: bool,

    #[arg(short, long, required = true)]
    /// Path to servo.
    servo_path: PathBuf,

    #[clap(flatten)]
    /// Benchmark flags
    benchmark: Benchmark,
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();
    let args = Args::parse();

    // Add tests here.
    // Because of async and stuff we have to do them manually and not put them into a vector
    testcase_run(&args, "Amazon", Box::new(amazon::test)).await?;
    testcase_run(&args, "Ebay", Box::new(ebay::test)).await?;
    testcase_run(&args, "Pinduoduo", Box::new(pinduoduo::test)).await?;
    testcase_run(&args, "TaoBao", Box::new(taobao::test)).await?;

    Ok(())
}

/// Run a whole testcase, start the process and waits for exit?
async fn testcase_run(
    args: &Args,
    name: &str,
    test: Box<impl AsyncFn(&WebDriver) -> Result<()>>,
) -> Result<()> {
    let result = run_single_test(args, name, test).await;
    match result {
        Ok(_) => info!("Test ran successful!"),
        Err(e) => error!("Webdriver test did not run succesful with error {}", e),
    }
    kill_servo()?;

    warn!("Look at your data and record it. Press <Enter> to do the next test");
    let stdin = stdin();

    // detecting keydown events
    for c in stdin.keys() {
        if let Ok(Key::Char('\n')) = c {
            break;
        }
    }
    info!("We will sleep for 3 seconds to make sure everything quit.");
    sleep(3).await;
    Ok(())
}

/// Runs a single test and exists the webdriver
async fn run_single_test(
    args: &Args,
    name: &str,
    test: Box<impl AsyncFn(&WebDriver) -> Result<()>>,
) -> Result<()> {
    let (mut process, process_args) = get_servo_cmd(args)?;
    info!("Process {:?}, args {:?}", process, process_args);
    info!("Runnign test {name}");
    info!("Starting process");
    let _servo = process
        .args(process_args)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()?;
    webdriver::sleep(5).await;
    info!("Connecting webdriver");
    let webdriver = webdriver::connect_webdriver_session().await?;
    info!("Running test");
    let test_run = test(&webdriver).await;
    webdriver::sleep(2).await;
    webdriver.quit().await?;
    test_run
}

/// Stop servo which is not the same as stopping the process given to use.
fn kill_servo() -> Result<()> {
    let pid = Command::new("pgrep").args(["-x", "servo"]).output()?;
    let pid_string = String::from_utf8(pid.stdout)?;
    let pid = pid_string.trim().parse()?;
    unsafe {
        libc::kill(pid, 9);
    }
    Ok(())
}

/// Construct the whole command arguments we use
fn get_servo_cmd(args: &Args) -> Result<(Command, VecDeque<String>)> {
    let servo_path = args
        .servo_path
        .as_path()
        .to_str()
        .ok_or(anyhow!("Could not find servo_path"))?;

    let mut all_args: VecDeque<String> = VecDeque::new();
    if args.proxychains {
        let proxychains = which("proxychains").context("Cannot find proxychains binary in path")?;
        all_args.push_back(proxychains.to_string_lossy().into_owned());
        all_args.push_back("-q".into());
    }

    if args.benchmark.heaptrack {
        all_args.push_back("heaptrack".into())
    } else {
        all_args.push_back("perf".into());
        all_args.push_back("record".into());
    }

    all_args.push_back(servo_path.into());

    all_args.push_back("--ignore-certificate-errors".into());
    all_args.push_back("--webdriver".into());

    let process = all_args.pop_front().unwrap();

    Ok((Command::new(process), all_args))
}
