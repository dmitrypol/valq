mod commands;
mod data_types;

use crate::commands::valq_cmd;
use crate::data_types::VALQ_TYPE;
use valkey_module::alloc::ValkeyAlloc;
use valkey_module::valkey_module;

valkey_module! {
    name: "valq",
    version: 1,
    allocator: (ValkeyAlloc, ValkeyAlloc),
    data_types: [VALQ_TYPE],
    commands: [
        ["valq", valq_cmd, "", 0, 0, 0],
    ],
}
