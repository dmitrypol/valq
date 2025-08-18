mod utils;

mod tests {
    use crate::utils;
    use anyhow::Context;
    use redis::RedisResult;
    use serial_test::serial;
    use std::thread;
    use std::time::Duration;

    #[test]
    #[serial]
    fn test_valq() -> anyhow::Result<()> {
        let port: u16 = 6479;
        let _guards = vec![
            utils::start_server_with_module("valq", port)
                .with_context(|| "failed to start valkey server")?,
        ];
        let mut con = utils::get_server_connection(port)
            .with_context(|| "failed to connect to valkey server")?;

        let test: Vec<String> = redis::cmd("valq").query(&mut con)?;
        assert_eq!(test.len(), 11);

        let test: Vec<String> = redis::cmd("valq").arg(&["help"]).query(&mut con)?;
        assert_eq!(test.len(), 11);

        // missing arguments
        for command in vec![
            "create", "delete", "update", "info", "purge", "push", "pop", "ack", "extend",
        ] {
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
        // extend message on invalid queue
        let test: RedisResult<String> = redis::cmd("valq")
            .arg(&["extend", "invalid-q", "invalid-id"])
            .query(&mut con);
        assert!(test.is_err());
        // create queue with invalid visibility timeout
        let test: RedisResult<String> = redis::cmd("valq")
            .arg(&["create", "invalid-q", "0"])
            .query(&mut con);
        assert!(test.is_err());
        // create queue with invalid max delivery attempts
        let test: RedisResult<String> = redis::cmd("valq")
            .arg(&["create", "invalid-q", "60", "0"])
            .query(&mut con);
        assert!(test.is_err());

        // create queue with custom visibility_timeout of 1 second and max_delivery_attempts 2
        let test: String = redis::cmd("valq")
            .arg(&["create", "q1", "1", "2"])
            .query(&mut con)?;
        assert_eq!(test, "created q");
        // duplicate queue name, should fail
        let test: RedisResult<String> = redis::cmd("valq").arg(&["create", "q1"]).query(&mut con);
        assert!(test.is_err());
        // create another queue with default visibility timeout and max_delivery_attempts
        let test: String = redis::cmd("valq").arg(&["create", "q2"]).query(&mut con)?;
        assert_eq!(test, "created q");

        // pop from empty queue
        let test: String = redis::cmd("valq").arg(&["pop", "q1"]).query(&mut con)?;
        assert_eq!(test, "");

        let test: String = redis::cmd("valq")
            .arg(&["push", "q1", "msg1"])
            .query(&mut con)?;
        assert_eq!(test, "1");
        let test: String = redis::cmd("valq")
            .arg(&["push", "q1", "msg2"])
            .query(&mut con)?;
        assert_eq!(test, "2");

        let test: RedisResult<String> = redis::cmd("valq")
            .arg(&["info", "invalid-q"])
            .query(&mut con);
        assert!(test.is_err());
        let test: Vec<String> = redis::cmd("valq").arg(&["info", "q1"]).query(&mut con)?;
        assert_eq!(
            test,
            [
                "delayed_msgs",
                "0",
                "dlq_msgs",
                "0",
                "id_sequence",
                "2",
                "max_delivery_attempts",
                "2",
                "msgs",
                "2",
                "visibility_timeout",
                "1"
            ]
        );
        let test: Vec<String> = redis::cmd("valq").arg(&["pop", "q1"]).query(&mut con)?;
        assert_eq!(test, ["body", "msg1", "id", "1"]);
        let test: Vec<String> = redis::cmd("valq").arg(&["pop", "q1"]).query(&mut con)?;
        assert_eq!(test, ["body", "msg2", "id", "2"]);
        // now q has no visible messages, so pop should return empty
        let test: Vec<String> = redis::cmd("valq").arg(&["pop", "q1"]).query(&mut con)?;
        assert_eq!(test, [""]);
        let test: Vec<String> = redis::cmd("valq").arg(&["info", "q1"]).query(&mut con)?;
        // TODO - exclude messages with timeout_at and delivery_attempts
        assert_eq!(
            test,
            [
                "delayed_msgs",
                "0",
                "dlq_msgs",
                "0",
                "id_sequence",
                "2",
                "max_delivery_attempts",
                "2",
                "msgs",
                "2",
                "visibility_timeout",
                "1"
            ]
        );

        let test: String = redis::cmd("valq")
            .arg(&["extend", "q1", "1", "1"])
            .query(&mut con)?;
        assert_eq!(test, "extend");
        // extend message with invalid id
        let test: RedisResult<String> = redis::cmd("valq")
            .arg(&["extend", "q1", "invalid-id", "100"])
            .query(&mut con);
        assert!(test.is_err());
        // extend message with invalid timeout
        let test: RedisResult<String> = redis::cmd("valq")
            .arg(&["extend", "q1", "1", "invalid-timeout"])
            .query(&mut con);
        assert!(test.is_err());
        // extend message with too large timeout, greater than 43_200 seconds (12 hours)
        let test: RedisResult<String> = redis::cmd("valq")
            .arg(&["extend", "q1", "1", "43201"])
            .query(&mut con);
        assert!(test.is_err());

        // sleep for messages to become visible again after queue visibility_timeout of 1 second
        thread::sleep(Duration::from_millis(1001));
        let test: Vec<String> = redis::cmd("valq").arg(&["pop", "q1"]).query(&mut con)?;
        assert_eq!(test, ["body", "msg1", "id", "1"]);
        let test: Vec<String> = redis::cmd("valq").arg(&["pop", "q1"]).query(&mut con)?;
        assert_eq!(test, ["body", "msg2", "id", "2"]);
        // now q has no visible messages, so pop should return empty
        let test: Vec<String> = redis::cmd("valq").arg(&["pop", "q1"]).query(&mut con)?;
        assert_eq!(test, [""]);
        // sleep again but queue will still have no visible messages because max_delivery_attempts is 2
        thread::sleep(Duration::from_millis(1001));
        let test: Vec<String> = redis::cmd("valq").arg(&["pop", "q1"]).query(&mut con)?;
        assert_eq!(test, [""]);

        let test: Vec<String> = redis::cmd("valq").arg(&["info", "q1"]).query(&mut con)?;
        // TODO - why is there 1 message in msgs?
        assert_eq!(
            test,
            [
                "delayed_msgs",
                "0",
                "dlq_msgs",
                "2",
                "id_sequence",
                "2",
                "max_delivery_attempts",
                "2",
                "msgs",
                "1",
                "visibility_timeout",
                "1"
            ]
        );

        // purge messages from q1
        let test: String = redis::cmd("valq").arg(&["purge", "q1"]).query(&mut con)?;
        assert_eq!(test, "1");
        let test: String = redis::cmd("valq")
            .arg(&["purge", "q1", "dlq"])
            .query(&mut con)?;
        assert_eq!(test, "2");
        let test: String = redis::cmd("valq")
            .arg(&["purge", "q1", "dlq"])
            .query(&mut con)?;
        assert_eq!(test, "0");
        let test: String = redis::cmd("valq")
            .arg(&["purge", "q1", "delayed"])
            .query(&mut con)?;
        assert_eq!(test, "0");
        let test: String = redis::cmd("valq")
            .arg(&["purge", "q1", "delayed"])
            .query(&mut con)?;
        assert_eq!(test, "0");

        // create new message to ack
        redis::cmd("valq")
            .arg(&["push", "q1", "msg3"])
            .exec(&mut con)?;
        let test: String = redis::cmd("valq")
            .arg(&["ack", "q1", "3"])
            .query(&mut con)?;
        assert_eq!(test, "ack");
        // ack invalid message id
        let test: RedisResult<String> = redis::cmd("valq")
            .arg(&["ack", "q1", "invalid-id"])
            .query(&mut con);
        assert!(test.is_err());

        // push message to q1 with delay
        let test: String = redis::cmd("valq")
            .arg(&["push", "q1", "msg4", "1"])
            .query(&mut con)?;
        assert_eq!(test, "4");
        // pop message from q1, it should be delayed
        let test: Vec<String> = redis::cmd("valq").arg(&["pop", "q1"]).query(&mut con)?;
        assert_eq!(test, [""]);
        // sleep for 1 second for delayed message to become visible
        thread::sleep(Duration::from_millis(1001));
        let test: Vec<String> = redis::cmd("valq").arg(&["pop", "q1"]).query(&mut con)?;
        assert_eq!(test, ["body", "msg4", "id", "4"]);

        // update queue with custom visibility_timeout and max_delivery_attempts
        let test: String = redis::cmd("valq")
            .arg(&["update", "q2", "10", "10"])
            .query(&mut con)?;
        assert_eq!(test, "updated q");
        let test: Vec<String> = redis::cmd("valq").arg(&["info", "q2"]).query(&mut con)?;
        assert_eq!(
            test,
            [
                "delayed_msgs",
                "0",
                "dlq_msgs",
                "0",
                "id_sequence",
                "0",
                "max_delivery_attempts",
                "10",
                "msgs",
                "0",
                "visibility_timeout",
                "10"
            ]
        );
        // update queue with invalid visibility_timeout
        let test: RedisResult<String> = redis::cmd("valq")
            .arg(&["update", "q2", "0", "1"])
            .query(&mut con);
        assert!(test.is_err());
        // update queue with invalid max_delivery_attempts
        let test: RedisResult<String> = redis::cmd("valq")
            .arg(&["update", "q2", "1", "0"])
            .query(&mut con);
        assert!(test.is_err());
        // update invalid queue
        let test: RedisResult<String> = redis::cmd("valq")
            .arg(&["update", "invalid-q", "1", "1"])
            .query(&mut con);
        assert!(test.is_err());

        // delete q1 and q2
        let test: String = redis::cmd("valq").arg(&["delete", "q1"]).query(&mut con)?;
        assert_eq!(test, "deleted q");
        let test: String = redis::cmd("valq").arg(&["delete", "q2"]).query(&mut con)?;
        assert_eq!(test, "deleted q");

        redis::cmd("save").exec(&mut con)?;
        Ok(())
    }

    #[test]
    #[serial]
    fn test_valq2() -> anyhow::Result<()> {
        let port: u16 = 6479;
        let _guards = vec![
            utils::start_server_with_module("valq", port)
                .with_context(|| "failed to start valkey server")?,
        ];
        let mut con = utils::get_server_connection(port)
            .with_context(|| "failed to connect to valkey server")?;

        //TODO leave q1/q2 in previous test and run info to check if q1/q2 exist after restart
        /*
        let test: Vec<String> = redis::cmd("valq").arg(&["info", "q1"]).query(&mut con)?;
        assert_eq!(
            test,
            [
                "delayed_msgs",
                "0",
                "dlq_msgs",
                "0",
                "id_sequence",
                "4",
                "max_delivery_attempts",
                "2",
                "msgs",
                "1",
                "visibility_timeout",
                "1"
            ]
        );
        let test: Vec<String> = redis::cmd("valq").arg(&["info", "q2"]).query(&mut con)?;
        assert_eq!(test.len(), 12,);
         */

        redis::cmd("save").exec(&mut con)?;
        Ok(())
    }
}
