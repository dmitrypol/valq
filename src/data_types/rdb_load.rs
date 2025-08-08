use crate::structs::{ValqMsg, ValqType};
use std::os::raw::c_void;
use valkey_module::logging::log_notice;

pub(crate) extern "C" fn rdb_load(
    rdb: *mut valkey_module::RedisModuleIO,
    _encver: i32,
) -> *mut c_void {
    if rdb.is_null() {
        return std::ptr::null_mut();
    }
    let mut valq = ValqType::new();
    // save and load must be in the same order
    // load id_sequence
    valq.set_id_sequence(match valkey_module::load_unsigned(rdb) {
        Ok(tmp) => tmp,
        Err(_err) => return std::ptr::null_mut(),
    });
    // load the size of the msgs VecDeque
    let q_size = valkey_module::load_unsigned(rdb).unwrap_or(0) as usize;
    for _ in 0..q_size {
        // load each message in the msgs VecDeque
        let id = match valkey_module::load_unsigned(rdb) {
            Ok(tmp) => tmp,
            Err(_err) => return std::ptr::null_mut(),
        };
        let body = match valkey_module::load_string(rdb) {
            Ok(tmp) => tmp.to_string(),
            Err(_err) => return std::ptr::null_mut(),
        };
        valq.msgs_mut().push_back(ValqMsg::new(id, body));
    }
    // load the size of the msgs_in_flight HashMap
    let in_flight_size = valkey_module::load_unsigned(rdb).unwrap_or(0) as usize;
    for _ in 0..in_flight_size {
        // load each message in the msgs_in_flight HashMap
        let id = match valkey_module::load_unsigned(rdb) {
            Ok(tmp) => tmp,
            Err(_err) => return std::ptr::null_mut(),
        };
        let msg = match valkey_module::load_string(rdb) {
            Ok(tmp) => tmp.to_string(),
            Err(_err) => return std::ptr::null_mut(),
        };
        // insert into the msgs_in_flight HashMap
        valq.msgs_in_flight_mut().insert(id, msg);
    }
    log_notice(format!("rdb_load: {:?}", valq));
    Box::into_raw(Box::new(valq)) as *mut c_void
}
