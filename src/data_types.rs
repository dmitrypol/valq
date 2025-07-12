use std::os::raw::c_void;
use valkey_module::{RedisModuleTypeMethods, native_types::ValkeyType, raw};

#[derive(Debug)]
pub(crate) struct ValqType {
    data: String,
}
impl ValqType {
    pub(crate) fn new(data: String) -> Self {
        Self { data }
    }
    pub(crate) fn data(&self) -> &str {
        &self.data
    }
    pub(crate) fn set_data(&mut self, data: String) {
        self.data = data;
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
    let item = value.cast::<ValqType>();
    unsafe {
        let _ = Box::from_raw(item);
    }
}

extern "C" fn rdb_save(rdb: *mut raw::RedisModuleIO, value: *mut c_void) {
    let item = unsafe { &*value.cast::<ValqType>() };
    raw::save_string(rdb, item.data());
}

extern "C" fn rdb_load(rdb: *mut raw::RedisModuleIO, _encver: i32) -> *mut c_void {
    match raw::load_string(rdb) {
        Ok(data) => {
            let value = ValqType::new(data.to_string());
            Box::into_raw(Box::new(value)) as *mut c_void
        }
        Err(_) => std::ptr::null_mut(),
    }
}

/*

extern "C" fn mem_usage(value: *const c_void) -> usize {
    let item = unsafe { &*value.cast::<ValqType>() };
    item.data().len()
}

extern "C" fn free_effort(_key: *mut RedisModuleString, value: *const c_void) -> usize {
    let item = unsafe { &*value.cast::<ValqType>() };
    item.data().len()
}
*/
