use crate::structs::valq_msg::ValqMsg;
use crate::structs::valq_type::ValqType;
use std::os::raw::c_void;
use valkey_module::logging::log_notice;
use valkey_module::{RedisModuleIO, save_string, save_unsigned};

/// Saves the state of a `ValqType` instance to the Valkey database.
///
/// This function is called by the Valkey module to persist the state of a `ValqType`
/// instance. It serializes the fields of the `ValqType` and its associated messages
/// in a specific order to ensure compatibility with the corresponding load function.
///
/// # Arguments
/// * `rdb` - A pointer to the RedisModuleIO structure used for saving data.
/// * `value` - A pointer to the `ValqType` instance to be saved.
///
/// # Safety
/// This function uses unsafe code to dereference raw pointers. It ensures that
/// the pointers are not null before accessing the data.
pub(crate) extern "C" fn rdb_save(rdb: *mut RedisModuleIO, value: *mut c_void) {
    if value.is_null() || rdb.is_null() {
        return;
    }
    let item = unsafe { &*value.cast::<ValqType>() };
    // save and load must be in the same order
    save_valq_attributes(rdb, item);
    save_msgs_attributes(rdb, item);
    save_dlq_msgs_attributes(rdb, item);
    save_delayed_msgs_attributes(rdb, item);
    // log the saved item
    log_notice(format!("rdb_save: {:?}", item));
}

fn save_valq_attributes(rdb: *mut RedisModuleIO, item: &ValqType) {
    // save name
    save_string(rdb, item.name().as_str());
    // save id_sequence
    save_unsigned(rdb, *item.id_sequence());
    // save visibility_timeout
    save_unsigned(rdb, *item.visibility_timeout());
    // save max_delivery_attempts
    save_unsigned(rdb, *item.max_delivery_attempts());
    // save retention_period
    save_unsigned(rdb, *item.retention_period());
}

fn save_msgs_attributes(rdb: *mut RedisModuleIO, item: &ValqType) {
    // save the size of the msgs VecDeque
    save_unsigned(rdb, item.msgs().len() as u64);
    // save each message in the msgs VecDeque
    item.msgs().iter().for_each(|msg| {
        save_each_msg(rdb, msg);
    });
}

fn save_dlq_msgs_attributes(rdb: *mut RedisModuleIO, item: &ValqType) {
    // save the size of the dlq_msgs VecDeque
    save_unsigned(rdb, item.dlq_msgs().len() as u64);
    // save each message in the dlq_msgs VecDeque
    item.dlq_msgs().iter().for_each(|msg| {
        save_each_msg(rdb, msg);
    });
}

fn save_delayed_msgs_attributes(rdb: *mut RedisModuleIO, item: &ValqType) {
    // save the size of the delayed_msgs
    save_unsigned(rdb, item.delayed_msgs().len());
    // save each message in the delayed_msgs
    item.delayed_msgs()
        .members()
        .iter()
        .for_each(|(msg, score)| {
            // save the score for the delayed message
            save_unsigned(rdb, *score);
            // save the message itself
            save_each_msg(rdb, msg);
        });
}

fn save_each_msg(rdb: *mut RedisModuleIO, msg: &ValqMsg) {
    // save id
    save_unsigned(rdb, *msg.id());
    // save body
    save_string(rdb, msg.body().as_str());
    // if timeout_at is None, it will be saved as 0
    // if timeout_at is Some, it will be saved as the actual value
    save_unsigned(rdb, msg.timeout_at().unwrap_or(0));
    // save delivery_attempts
    save_unsigned(rdb, *msg.delivery_attempts());
}
