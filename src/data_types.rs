use std::collections::VecDeque;
use std::os::raw::c_void;
use valkey_module::logging::log_notice;
use valkey_module::{RedisModuleTypeMethods, native_types::ValkeyType, raw};

#[derive(Debug, Clone, Default)]
pub(crate) struct ValqMsg {
    id: u64,
    body: String,
}
impl ValqMsg {
    pub(crate) fn new(id: u64, body: String) -> Self {
        Self { id, body }
    }
    pub(crate) fn body(&self) -> &String {
        &self.body
    }
}

#[derive(Debug, Clone)]
pub(crate) struct ValqType {
    pub(crate) id_sequence: u64,
    data: VecDeque<ValqMsg>,
}
impl ValqType {
    pub(crate) fn new(data: VecDeque<ValqMsg>) -> Self {
        Self {
            id_sequence: 1,
            data,
        }
    }
    pub(crate) fn data(&self) -> &VecDeque<ValqMsg> {
        &self.data
    }
    pub(crate) fn data_mut(&mut self) -> &mut VecDeque<ValqMsg> {
        &mut self.data
    }
}

pub(crate) static VALQ_TYPE: ValkeyType = ValkeyType::new(
    "valq-type",
    1,
    RedisModuleTypeMethods {
        version: valkey_module::TYPE_METHOD_VERSION,
        rdb_load: Some(rdb_load),
        rdb_save: Some(rdb_save),
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

extern "C" fn rdb_save(rdb: *mut raw::RedisModuleIO, value: *mut c_void) {
    if value.is_null() || rdb.is_null() {
        return;
    }
    let item = unsafe { &*value.cast::<ValqType>() };
    // save and load must be in the same order
    // save id_sequence
    raw::save_unsigned(rdb, item.id_sequence);
    // save the size of the VecDeque
    raw::save_unsigned(rdb, item.data.len() as u64);
    // save each message in the VecDeque
    item.data.iter().for_each(|msg| {
        // save id
        raw::save_unsigned(rdb, msg.id);
        // save body
        raw::save_string(rdb, msg.body.as_str());
    });
    log_notice(format!("rdb_save: {:?}", item));
}

extern "C" fn rdb_load(rdb: *mut raw::RedisModuleIO, _encver: i32) -> *mut c_void {
    if rdb.is_null() {
        return std::ptr::null_mut();
    }
    let mut valq = ValqType::new(VecDeque::new());
    // save and load must be in the same order
    // load id_sequence
    valq.id_sequence = match raw::load_unsigned(rdb) {
        Ok(tmp) => tmp,
        Err(_err) => return std::ptr::null_mut(),
    };
    // load the size of the VecDeque
    let q_size = raw::load_unsigned(rdb).unwrap_or(0) as usize;
    for _ in 0..q_size {
        // load each message in the VecDeque
        let id = match raw::load_unsigned(rdb) {
            Ok(tmp) => tmp,
            Err(_err) => return std::ptr::null_mut(),
        };
        let body = match raw::load_string(rdb) {
            Ok(tmp) => tmp.to_string(),
            Err(_err) => return std::ptr::null_mut(),
        };
        valq.data_mut().push_back(ValqMsg::new(id, body));
    }
    log_notice(format!("rdb_load: {:?}", valq));
    Box::into_raw(Box::new(valq)) as *mut c_void
}
