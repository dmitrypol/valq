cargo fmt
cargo b
valkey-server --loadmodule ./target/debug/libvalq.dylib --daemonize yes #--enable-module-command yes --save ""
sleep 1
#valkey-cli -3 flushall
valkey-cli -3 valq
valkey-cli -3 valq help
echo "--------------- args not passed:"
valkey-cli -3 valq push
valkey-cli -3 valq pop
valkey-cli -3 valq len
echo "--------------- push/pop messages to/from q1:"
valkey-cli -3 valq push q1 msq1a
valkey-cli -3 valq push q1 msq1b
valkey-cli -3 valq push q1 msq1c
valkey-cli -3 valq len q1
valkey-cli -3 valq pop q1
valkey-cli -3 valq pop q1
valkey-cli -3 valq pop q1
valkey-cli -3 valq pop q1
valkey-cli -3 valq len q1
echo "--------------- test q2:"
valkey-cli -3 valq push q2 msq2a
valkey-cli -3 valq pop q2
valkey-cli -3 valq len q2
echo "--------------- check invalid queue":
valkey-cli -3 valq pop invalid-queue
valkey-cli -3 valq len invalid
echo "--------------- pop from queue from previous test:"
valkey-cli -3 valq pop leave
echo "--------------- push to load in the next test:"
valkey-cli -3 valq push leave behind
valkey-cli -3 valq push leave behind
valkey-cli -3 valq len leave
valkey-cli -3 keys "*"
valkey-cli -3 save
valkey-cli shutdown
