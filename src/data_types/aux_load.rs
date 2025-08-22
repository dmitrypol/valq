use crate::GLOBAL_Q_LIST;
use std::os::raw::c_int;
use valkey_module::logging::log_notice;
use valkey_module::{RedisModuleIO, Status, load_string, load_unsigned};

pub(crate) extern "C" fn aux_load(rdb: *mut RedisModuleIO, _encver: c_int, _when: c_int) -> c_int {
    if rdb.is_null() {
        return Status::Err as i32;
    }
    let q_list_len = load_unsigned(rdb).unwrap_or(0);
    log_notice(format!("aux_load q_list_len: {}", q_list_len));
    let mut q_list = match GLOBAL_Q_LIST.write() {
        Ok(tmp) => tmp,
        Err(err) => {
            log_notice(format!("aud_load err: {}", err));
            return Status::Err as i32;
        }
    };
    for _ in 0..q_list_len {
        match load_string(rdb) {
            Ok(q_name) => {
                q_list.insert(q_name.to_string());
            }
            Err(err) => {
                log_notice(format!("aud_load q_name err: {}", err));
                Status::Err as i32;
            }
        }
    }
    log_notice(format!("aud_load q_list: {:?}", q_list));
    Status::Ok as i32
}
