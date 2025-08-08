use crate::structs::ValqType;
use std::os::raw::c_void;
use valkey_module::{RedisModuleTypeMethods, native_types::ValkeyType};

mod rdb_load;
mod rdb_save;

pub(crate) static VALQ_TYPE: ValkeyType = ValkeyType::new(
    "valq-type",
    1,
    RedisModuleTypeMethods {
        version: valkey_module::TYPE_METHOD_VERSION,
        rdb_load: Some(rdb_load::rdb_load),
        rdb_save: Some(rdb_save::rdb_save),
        aof_rewrite: None,
        free: Some(free),
        mem_usage: None,
        digest: None,
        aux_load: None,
        aux_save: None,
        aux_save_triggers: 0,
        free_effort: None,
        unlink: None,
        copy: None,
        defrag: None,
        free_effort2: None,
        unlink2: None,
        copy2: None,
        mem_usage2: None,
        aux_save2: None,
    },
);

extern "C" fn free(value: *mut c_void) {
    if value.is_null() {
        return;
    }
    unsafe {
        let _ = Box::from_raw(value.cast::<ValqType>());
    }
}
