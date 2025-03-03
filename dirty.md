/run/current-system/sw/bin/docker

/workspaces/edgeless/target/debug/edgeless_inabox -t
/workspaces/edgeless/target/debug/edgeless_cli -t cli.toml

export WASMTIME_LOG=trace

export RUST_LOG=debug,h2=warn,cranelift_codegen=warn,wasmtime_cranelift=warn,tower=warn,edgeless_orc::orchestrator=info,edgeless_bal=info

/workspaces/edgeless/target/debug/edgeless_inabox


    

/workspaces/edgeless/target/debug/edgeless_cli workflow start /workspaces/edgeless/examples/http_ingress/workflow.json

for i in $(seq 10); do echo $i; curl -H "Host: demo.edgeless.com" -XPOST http://127.0.0.1:7035/hello; echo ''; sleep 0; done