use crate::GLOBAL_Q_LIST;
use crate::structs::valq_type::ValqType;
use std::os::raw::c_void;
use valkey_module::logging::log_notice;
use valkey_module::{RedisModuleTypeMethods, native_types::ValkeyType, raw};

mod aux_load;
mod aux_save;
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
        aux_load: Some(aux_load::aux_load),
        aux_save: Some(aux_save::aux_save),
        aux_save_triggers: raw::Aux::Before as i32,
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
        let valq_type = value.cast::<ValqType>();
        let q_name = (*valq_type).name();
        // update GLOBAL_Q_LIST on free which happens for flushdb or del
        match GLOBAL_Q_LIST.write() {
            Ok(mut q_list) => {
                q_list.remove(q_name);
            }
            Err(err) => {
                log_notice(format!("free err: {}", err));
            }
        };
        // Convert the raw pointer back to a Box to properly deallocate the memory.
        let _ = Box::from_raw(valq_type);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn free_null_pointer() {
        free(std::ptr::null_mut());
        // ensuring no panic occurs
    }

    #[test]
    #[ignore]
    fn free_non_null_pointer() {
        let valq = Box::new(ValqType::new("", None, None));
        let raw_ptr = Box::into_raw(valq);
        free(raw_ptr.cast());
        // ensuring no memory leaks or panics occur
    }
}
