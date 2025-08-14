mod commands;
mod data_types;
mod structs;
mod utils;

use crate::commands::valq_cmd;
use crate::data_types::VALQ_TYPE;
use valkey_module::alloc::ValkeyAlloc;
use valkey_module::valkey_module;

static VISIBILITY_TIMEOUT_DEFAULT: u64 = 30;
static VISIBILITY_TIMEOUT_MAX: u64 = 43_200; // 12 hours
static DELIVERY_ATTEMPTS_DEFAULT: u64 = 5;
static DELIVERY_ATTEMPTS_MAX: u64 = 20;

valkey_module! {
    name: "valq",
    version: 1,
    allocator: (ValkeyAlloc, ValkeyAlloc),
    data_types: [VALQ_TYPE],
    commands: [
        ["valq", valq_cmd, "", 0, 0, 0],
    ],
}
