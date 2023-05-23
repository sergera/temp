SHELL:=/bin/bash

.PHONY: help

KERNEL_NAME := $(shell uname -s)
ifeq ($(KERNEL_NAME),Linux)
    OPEN := xdg-open
else ifeq ($(KERNEL_NAME),Darwin)
    OPEN := open
else
    $(error unsupported system: $(KERNEL_NAME))
endif

help: ## Print this help
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | sort | awk 'BEGIN {FS = ":.*?## "}; {printf "\033[36m%-30s\033[0m %s\n", $$1, $$2}'

deploy: ## deploy a contract locally
	forge create --rpc-url http://127.0.0.1:8545 --constructor-args 0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266 --private-key 0x2a871d0798f97d79848a013d4936a73bf4cc922c825d33c1cf7073dff6d409c6 src/Escrow.sol:Escrow

deploy_tokens: ## deploy a ERC20 token locally
	forge create --rpc-url http://127.0.0.1:8545 --constructor-args "TestToken" "TEST" --private-key 0x2a871d0798f97d79848a013d4936a73bf4cc922c825d33c1cf7073dff6d409c6 "lib/openzeppelin-contracts/contracts/token/ERC20/ERC20.sol":ERC20

transfer_tokens: ## transfer ERC20 to address
