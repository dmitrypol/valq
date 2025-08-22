use crate::GLOBAL_Q_LIST;
use valkey_module::{ValkeyResult, ValkeyValue};

pub(crate) fn list() -> ValkeyResult {
    let q_list = GLOBAL_Q_LIST.read()?;
    let output: Vec<ValkeyValue> = q_list.iter().map(|s| s.into()).collect();
    Ok(output.into())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_list() {
        let test = list();
        assert!(test.is_ok());
        match test.unwrap() {
            ValkeyValue::Array(tmp) => {
                assert_eq!(tmp.len(), 0);
            }
            _ => {}
        }
    }
}
