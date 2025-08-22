mod commands;
mod data_types;
mod structs;
mod utils;

use crate::commands::valq_cmd;
use crate::data_types::VALQ_TYPE;
use crate::utils::{retention_period_gc, valid_server_version};
use std::collections::HashSet;
use std::sync::{LazyLock, RwLock};
use std::thread;
use std::time::Duration;
use valkey_module::alloc::ValkeyAlloc;
use valkey_module::{Context, Status, ThreadSafeContext, ValkeyString, valkey_module};

static MIN_VALID_SERVER_VERSION: &[i32; 3] = &[7, 2, 8];
static VISIBILITY_TIMEOUT_DEFAULT: u64 = 30;
static VISIBILITY_TIMEOUT_MAX: u64 = 43_200; // 12 hours
static DELIVERY_ATTEMPTS_DEFAULT: u64 = 5;
static DELIVERY_ATTEMPTS_MAX: u64 = 20;
static RETENTION_PERIOD_DEFAULT: u64 = 86_400; // 1 day
static RETENTION_PERIOD_MAX: u64 = 604_800; // 7 days
static RETENTION_PERIOD_MIN: u64 = 60;
static GLOBAL_Q_LIST: LazyLock<RwLock<HashSet<String>>> =
    LazyLock::new(|| RwLock::new(HashSet::new()));

fn preload(ctx: &Context, _args: &[ValkeyString]) -> Status {
    let ver = ctx.get_server_version().expect("can't get_server_version");
    if !valid_server_version(ver) {
        ctx.log_notice(format!("min valid server version {:?}", MIN_VALID_SERVER_VERSION).as_str());
        Status::Err
    } else {
        Status::Ok
    }
}

fn init(_ctx: &Context, _args: &[ValkeyString]) -> Status {
    thread::spawn(move || {
        let ts_ctx = ThreadSafeContext::new();
        loop {
            let ctx_guard = ts_ctx.lock();
            retention_period_gc::run(&ctx_guard);
            drop(ctx_guard);
            thread::sleep(Duration::from_secs(30));
        }
    });
    Status::Ok
}

valkey_module! {
    name: "valq",
    version: 1,
    allocator: (ValkeyAlloc, ValkeyAlloc),
    data_types: [VALQ_TYPE],
    preload: preload,
    init: init,
    commands: [
        ["valq", valq_cmd, "", 0, 0, 0],
    ],
}
