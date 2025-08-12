use crate::structs::valq_type::ValqType;
use std::os::raw::c_void;
use valkey_module::logging::log_notice;

pub(crate) extern "C" fn rdb_save(rdb: *mut valkey_module::RedisModuleIO, value: *mut c_void) {
    if value.is_null() || rdb.is_null() {
        return;
    }
    let item = unsafe { &*value.cast::<ValqType>() };
    // save and load must be in the same order
    // save id_sequence
    valkey_module::save_unsigned(rdb, *item.id_sequence());
    // save the size of the msgs VecDeque
    valkey_module::save_unsigned(rdb, item.msgs().len() as u64);
    // save each message in the msgs VecDeque
    item.msgs().iter().for_each(|msg| {
        // save id
        valkey_module::save_unsigned(rdb, *msg.id());
        // save body
        valkey_module::save_string(rdb, msg.body().as_str());
        // if timeout_at is None, it will be saved as 0
        // if timeout_at is Some, it will be saved as the actual value
        valkey_module::save_unsigned(rdb, msg.timeout_at().unwrap_or(0));
    });
    // log the saved item
    log_notice(format!("rdb_save: {:?}", item));
}
