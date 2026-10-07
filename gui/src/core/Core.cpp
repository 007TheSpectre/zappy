/*
** EPITECH PROJECT, 2026
** Zapfrites
** File description:
** Core
*/

#include "Core.hpp"
#include <Keyboard.hpp>
#include <Rectangle.hpp>
#include <Window.hpp>
#include <chrono>
#include <future>
#include <raylib.h>
#include <utility>
#include <vector>

namespace zappy {

    void Core::updateSelectedPlayer(std::vector<Player>& players) {
        if (!_renderer.getSelectedPlayerId())
            return;
        int selectedId = *_renderer.getSelectedPlayerId();
        for (int i = 0; i < players.size(); i++) {
            if (players[i].getId() == selectedId) {
                _renderer.setSelectedPlayerId(selectedId);
                return;
            }
        }
    }

    void Core::handleSelectTile() {
        std::vector<std::vector<raylib::Rectangle>> Tiles = _renderer.getDrawnTiles();
        std::pair<int, int>                         newPos;

        for (int y = 0; y < Tiles.size(); y++) {
            for (int x = 0; x < Tiles[y].size(); x++) {
                if (Tiles[y][x].CheckCollision(GetMousePosition()) && IsMouseButtonPressed(MOUSE_LEFT_BUTTON)) {
                    newPos = {x, y};
                    _renderer.setSelectedTile(newPos);
                }
            }
        }
    }

    void Core::handleEvents(std::vector<Player> players) {
        raylib::Window& window      = _renderer.getWindow();
        Colors&         colors      = _renderer.getColors();
        static int      indexPlayer = 0;

        if (raylib::Keyboard::IsKeyPressed(KEY_F3))
            _isFpsDisplayed = !_isFpsDisplayed;
        if (raylib::Keyboard::IsKeyPressed(KEY_F11)) {
            _isFullscreen = !_isFullscreen;
            window.SetFullscreen(_isFullscreen);
        }
        if (raylib::Keyboard::IsKeyPressed(KEY_F2))
            _isCursorDefault = !_isCursorDefault;
        if ((raylib::Keyboard::IsKeyPressed(KEY_KP_ADD) || raylib::Keyboard::IsKeyPressed(KEY_KP_SUBTRACT)) && !players.empty()) {
            if (raylib::Keyboard::IsKeyPressed(KEY_KP_ADD)) {
                indexPlayer = (indexPlayer + 1) % players.size();
                _renderer.setSelectedPlayerId(players[indexPlayer].getId());
            }
            if (raylib::Keyboard::IsKeyPressed(KEY_KP_SUBTRACT)) {
                indexPlayer = (indexPlayer - 1 + players.size()) % players.size();
                _renderer.setSelectedPlayerId(players[indexPlayer].getId());
            }
        }
        colors.handleEvents();
        handleSelectTile();
    }

    int Core::Run(Socket& socket) {
        _renderer.setupColors();
        _renderer.createWindow(_isFullscreen);
        Server server(socket);
        server.getInfo();
        server.updateResources();
        auto              lastUpdate = std::chrono::steady_clock::now();
        std::future<void> updateFuture;
        raylib::Window&   window = _renderer.getWindow();
        while (!window.ShouldClose()) {
            const auto now        = std::chrono::steady_clock::now();
            const auto elapsedMs  = std::chrono::duration_cast<std::chrono::milliseconds>(now - lastUpdate).count();
            const auto t          = server.world().getTimeReference();
            const auto intervalMs = (t > 0) ? static_cast<long long>(1000.0 / t) : 1000LL;
            if (elapsedMs >= intervalMs && !updateFuture.valid()) {
                updateFuture = std::async(std::launch::async, [&server, this]() {
                    server.handleServerUpdate();
                    server.updateResources();
                    updateSelectedPlayer(server.players().getPlayer());
                    _renderer.updateBroadcasters(server.players().takeBroadcasters(), server);
                    _renderer.updateIncreasers(server.players().takeIncreasers(), server);
                    server.players().clearBroadcasters();
                    server.players().clearIncreasers();
                });
                lastUpdate   = std::chrono::steady_clock::now();
            }
            if (updateFuture.valid()) {
                updateFuture.get();
            }
            handleEvents(server.players().getPlayer());
            _renderer.displayAll(server, _isFpsDisplayed, _isCursorDefault);
        }
        _renderer.close();
        return 0;
    }
}
