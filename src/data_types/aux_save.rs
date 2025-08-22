use crate::GLOBAL_Q_LIST;
use std::os::raw::c_int;
use valkey_module::logging::log_notice;
use valkey_module::{RedisModuleIO, save_string, save_unsigned};

pub(crate) extern "C" fn aux_save(rdb: *mut RedisModuleIO, _when: c_int) {
    if rdb.is_null() {
        return;
    }
    let q_list = match GLOBAL_Q_LIST.read() {
        Ok(guard) => guard.clone(),
        Err(_) => {
            log_notice("aux_save err");
            return;
        }
    };

    let q_list_len = q_list.len() as u64;
    save_unsigned(rdb, q_list_len);
    log_notice(format!("aux_save q_list_len: {}", q_list_len));

    for q_name in &q_list {
        save_string(rdb, q_name);
        log_notice(format!("aux_save q_name: {}", q_name));
    }
}
