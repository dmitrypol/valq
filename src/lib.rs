mod commands;
mod data_types;
mod structs;
mod utils;

use crate::commands::valq_cmd;
use crate::data_types::VALQ_TYPE;
use crate::utils::valid_server_version;
use valkey_module::alloc::ValkeyAlloc;
use valkey_module::{Context, Status, ValkeyString, valkey_module};

static MIN_VALID_SERVER_VERSION: &[i32; 3] = &[7, 2, 8];
static VISIBILITY_TIMEOUT_DEFAULT: u64 = 30;
static VISIBILITY_TIMEOUT_MAX: u64 = 43_200; // 12 hours
static DELIVERY_ATTEMPTS_DEFAULT: u64 = 5;
static DELIVERY_ATTEMPTS_MAX: u64 = 20;

fn preload(ctx: &Context, _args: &[ValkeyString]) -> Status {
    let ver = ctx.get_server_version().expect("can't get_server_version");
    if !valid_server_version(ver) {
        ctx.log_notice(format!("min valid server version {:?}", MIN_VALID_SERVER_VERSION).as_str());
        Status::Err
    } else {
        Status::Ok
    }
}
valkey_module! {
    name: "valq",
    version: 1,
    allocator: (ValkeyAlloc, ValkeyAlloc),
    data_types: [VALQ_TYPE],
        preload: preload,
    commands: [
        ["valq", valq_cmd, "", 0, 0, 0],
    ],
}
