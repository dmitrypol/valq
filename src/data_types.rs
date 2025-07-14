use std::collections::VecDeque;
use std::os::raw::c_void;
use valkey_module::logging::log_notice;
use valkey_module::{RedisModuleTypeMethods, native_types::ValkeyType, raw};

#[derive(Debug, Clone)]
pub(crate) struct ValqType {
    data: VecDeque<String>,
}
impl ValqType {
    pub(crate) fn new(data: VecDeque<String>) -> Self {
        Self { data }
    }
    pub(crate) fn data(&self) -> &VecDeque<String> {
        &self.data
    }
    pub(crate) fn data_mut(&mut self) -> &mut VecDeque<String> {
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
    // save the size of the VecDeque
    raw::save_unsigned(rdb, item.data.len() as u64);
    // save each string in the VecDeque
    item.data.iter().for_each(|str| {
        raw::save_string(rdb, str.as_str());
    });
    log_notice(&format!("rdb_save: {:?}", item));
}

extern "C" fn rdb_load(rdb: *mut raw::RedisModuleIO, _encver: i32) -> *mut c_void {
    if rdb.is_null() {
        return std::ptr::null_mut();
    }
    let mut valq = ValqType::new(VecDeque::new());
    // load the size of the VecDeque
    let q_size = match raw::load_unsigned(rdb) {
        Ok(tmp) => tmp as usize,
        Err(_) => return std::ptr::null_mut(),
    };
    for count in 0..q_size {
        // load each string in the VecDeque
        match raw::load_string(rdb) {
            Ok(tmp) => valq.data_mut().push_back(tmp.to_string()),
            Err(err) => {
                log_notice(&format!("rdb_load error: {}, count: {}", err, count));
                return std::ptr::null_mut();
            }
        };
    }
    log_notice(&format!("rdb_load: {:?}", valq));
    Box::into_raw(Box::new(valq)) as *mut c_void
}
