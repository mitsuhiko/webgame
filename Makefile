client:
	@cd webgame_client && yarn && yarn run start:dev
.PHONY: client

server:
	@cd webgame_server && RUST_LOG=debug cargo run
.PHONY: server
