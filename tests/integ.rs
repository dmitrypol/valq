mod utils;

use anyhow::Context;
use redis::RedisResult;

#[test]
fn test_valq() -> anyhow::Result<()> {
    let port: u16 = 6479;
    let _guards = vec![
        utils::start_server_with_module("valq", port)
            .with_context(|| "failed to start valkey server")?,
    ];
    let mut con =
        utils::get_server_connection(port).with_context(|| "failed to connect to valkey server")?;

    let test: Vec<String> = redis::cmd("valq").query(&mut con)?;
    assert_eq!(test.len(), 7);

    let test: Vec<String> = redis::cmd("valq").arg(&["info"]).query(&mut con)?;
    assert_eq!(test.len(), 7);

    // missing arguments
    for command in vec!["create", "delete", "push", "pop", "ack", "len"] {
        let test: RedisResult<String> = redis::cmd("valq").arg(&[command]).query(&mut con);
        assert!(test.is_err());
    }

    // push to invalid queue
    let test: RedisResult<String> = redis::cmd("valq")
        .arg(&["push", "invalid-q", "invalid-msg"])
        .query(&mut con);
    assert!(test.is_err());
    // pop from invalid queue
    let test: RedisResult<String> = redis::cmd("valq")
        .arg(&["pop", "invalid-q"])
        .query(&mut con);
    assert!(test.is_err());
    // ack on invalid queue
    let test: RedisResult<String> = redis::cmd("valq")
        .arg(&["ack", "invalid-q", "invalid-id"])
        .query(&mut con);
    assert!(test.is_err());

    let test: String = redis::cmd("valq").arg(&["create", "q1"]).query(&mut con)?;
    assert_eq!(test, "created q");
    let test: RedisResult<String> = redis::cmd("valq").arg(&["create", "q1"]).query(&mut con);
    assert!(test.is_err());
    let test: String = redis::cmd("valq").arg(&["create", "q2"]).query(&mut con)?;
    assert_eq!(test, "created q");

    // pop from empty queue
    let test: String = redis::cmd("valq").arg(&["pop", "q1"]).query(&mut con)?;
    assert_eq!(test, "");

    let test: String = redis::cmd("valq")
        .arg(&["push", "q1", "m1"])
        .query(&mut con)?;
    assert_eq!(test, "1");
    let test: String = redis::cmd("valq")
        .arg(&["push", "q1", "m2"])
        .query(&mut con)?;
    assert_eq!(test, "2");

    let test: String = redis::cmd("valq").arg(&["len", "q1"]).query(&mut con)?;
    assert_eq!(test, "2");
    let test: Vec<String> = redis::cmd("valq").arg(&["pop", "q1"]).query(&mut con)?;
    assert_eq!(test, ["body", "m1", "id", "1"]);
    let test: Vec<String> = redis::cmd("valq").arg(&["pop", "q1"]).query(&mut con)?;
    assert_eq!(test, ["body", "m2", "id", "2"]);
    let test: String = redis::cmd("valq").arg(&["len", "q1"]).query(&mut con)?;
    assert_eq!(test, "0");

    let test: String = redis::cmd("valq")
        .arg(&["ack", "q1", "1"])
        .query(&mut con)?;
    assert_eq!(test, "ack");
    let test: String = redis::cmd("valq")
        .arg(&["ack", "q1", "2"])
        .query(&mut con)?;
    assert_eq!(test, "ack");
    // ack invalid message id
    let test: RedisResult<String> = redis::cmd("valq")
        .arg(&["ack", "q1", "invalid-id"])
        .query(&mut con);
    assert!(test.is_err());

    let test: String = redis::cmd("valq").arg(&["delete", "q1"]).query(&mut con)?;
    assert_eq!(test, "deleted q");
    let test: String = redis::cmd("valq").arg(&["delete", "q2"]).query(&mut con)?;
    assert_eq!(test, "deleted q");

    Ok(())
}
