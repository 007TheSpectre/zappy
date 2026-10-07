SERVER_BIN 	:= zappy_server
GUI_BIN		:= zappy_gui
AI_BIN		:= zappy_ai

SHELL := /usr/bin/env bash

.PHONY = all clean fclean re $(SERVER_BIN) $(AI_BIN) $(GUI_BIN) clean_serv server gui ai

all: server ai gui

server: $(SERVER_BIN)

$(SERVER_BIN): clean_serv
	@echo "Building server..."
	@cargo build --release --manifest-path ./server/Cargo.toml
	@cp ./server/target/release/$(SERVER_BIN) ./

clean_serv:
	@echo "Cleaning server..."
	@cargo clean --manifest-path ./server/Cargo.toml

gui: $(GUI_BIN)

$(GUI_BIN):
	@echo "Building GUI..."
	@cd gui && cmake -S . -B build && cmake --build build --config Release
	@cp ./gui/build/$(GUI_BIN) ./

ai: $(AI_BIN)

$(AI_BIN):
	@echo "Building AI..."
	@if [ ! -d "./ai/.venv" ]; then \
		python3 -m venv ./ai/.venv; \
	fi
	@./ai/.venv/bin/pip install -r ./ai/requirements.txt > ai/venv_install.log 2>&1
	@cp ./ai/entry.sh ./$(AI_BIN)
	@chmod +x ./$(AI_BIN)

clean_ai:
	@echo "Cleaning AI..."
	@rm -f $(AI_BIN)

clean: clean_serv clean_ai

fclean: clean
	@echo "Removing binaries..."
	@rm -f $(SERVER_BIN) $(GUI_BIN) $(AI_BIN)

re: fclean all
