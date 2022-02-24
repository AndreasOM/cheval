flame:
	sudo CARGO_PROFILE_RELEASE_DEBUG=true cargo flamegraph -- --config ../cheval-example-configs/alpha_stripes --frames 1200 --window-mode RGB_A
	mv flamegraph.svg flamegraphs/flamegraph-$(date -u +"%Y-%m-%dT%H%M%SZ").svg

