client:
	@cd webgame_client && yarn && yarn run start:dev
.PHONY: client

server:
	@cd webgame_server && RUST_LOG=debug cargo run
.PHONY: server

server-reload:
	@cd webgame_server && RUST_LOG=debug systemfd --no-pid -s http::8002 -- cargo watch -x run
.PHONY: server-reload
