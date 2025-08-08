cargo fmt
cargo b
cargo test --all --features enable-system-alloc
# run via valkey-cli
valkey-server --loadmodule ./target/debug/libvalq.dylib --daemonize yes #--enable-module-command yes --save ""
sleep 1
#valkey-cli -3 flushall
valkey-cli -3 valq
# more tests here
valkey-cli -3 keys "*"
valkey-cli -3 save
valkey-cli shutdown
