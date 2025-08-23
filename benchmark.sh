NUM_MSGS=10000
valkey-server --loadmodule ./target/debug/libvalq.dylib --daemonize yes #--enable-module-command yes --save ""
sleep 1
valkey-cli flushall
valkey-cli valq create q1
echo "---------------- valq messages benchmark ---------------- "
valkey-benchmark -q -n $NUM_MSGS -c 50 valq push q1 m1
valkey-benchmark -q -n $NUM_MSGS -c 50 valq pop q1
valkey-cli info memory | grep used_memory_dataset:
valkey-cli valq purge q1
echo "---------------- valq delayed messages benchmark ----------------"
valkey-benchmark -q -n $NUM_MSGS -c 50 valq push q1 m1 1
sleep 1
valkey-benchmark -q -n $NUM_MSGS -c 50 valq pop q1
valkey-cli info memory | grep used_memory_dataset:
valkey-cli valq delete q1
echo "---------------- lpush / rpop benchmark ----------------"
valkey-benchmark -q -n $NUM_MSGS -c 50 lpush l1 m1
valkey-benchmark -q -n $NUM_MSGS -c 50 rpop l1
valkey-cli info memory | grep used_memory_dataset:
echo "---------------- zadd / zpopmin benchmark ----------------"
valkey-benchmark -q -n $NUM_MSGS -c 50 zadd z1 1 m1
valkey-benchmark -q -n $NUM_MSGS -c 50 zpopmin z1
valkey-cli info memory | grep used_memory_dataset:
valkey-cli save
valkey-cli shutdown
echo "---------------- done ----------------"
