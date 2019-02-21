CARGO_TARGET_DIR := $(CURDIR)/../target/wasm32-unknown-unknown
STATIC_DIR := $(CURDIR)/static
RELEASE_DIR := ${CARGO_TARGET_DIR}/release
DEBUG_DIR := ${CARGO_TARGET_DIR}/debug
DEPLOY_DIR ?= $(CURDIR)/build

default: deploy

deploy:
	@cargo web build --release
	@rm -rf ${DEPLOY_DIR} && mkdir ${DEPLOY_DIR}
	@cp ${RELEASE_DIR}/packer.wasm ${DEPLOY_DIR}
	@cp ${RELEASE_DIR}/packer.js ${DEPLOY_DIR}
	@cp ${RELEASE_DIR}/app.wasm ${DEPLOY_DIR}
	@cp ${RELEASE_DIR}/app.js ${DEPLOY_DIR}
	@cp ${RELEASE_DIR}/worker.wasm ${DEPLOY_DIR}
	@cp ${STATIC_DIR}/index.html ${DEPLOY_DIR}
	@cp ${STATIC_DIR}/style.css ${DEPLOY_DIR}
	@cp ${STATIC_DIR}/TrackballControls.js ${DEPLOY_DIR}

dev:
	@cargo web build --bin packer
	@cp ${DEBUG_DIR}/packer.wasm ${STATIC_DIR}
	@cp ${DEBUG_DIR}/packer.js ${STATIC_DIR}
	@cargo web start --bin app

clean:
	@cargo clean
	@rm -rf ${DEPLOY_DIR}
