cargo fmt
cargo b
valkey-server --loadmodule ./target/debug/libvalq.dylib --daemonize yes #--enable-module-command yes --save ""
sleep 1
#valkey-cli -3 flushall
valkey-cli -3 valq
valkey-cli -3 valq help
# check args not passed - ERR wrong number of arguments
valkey-cli -3 valq push
valkey-cli -3 valq pop
# check invalid queue
valkey-cli -3 valq pop invalid-queue
# pass args
valkey-cli -3 valq push q1 msq1a
# push another message to the same queue
valkey-cli -3 valq push q1 msq1b
valkey-cli -3 valq pop q1
# push to another queue
valkey-cli -3 valq push q2 msq2a
valkey-cli -3 valq pop q2
# push to another queue to load on startup
valkey-cli -3 valq push leave behind
#valkey-cli -3 save
valkey-cli shutdown
