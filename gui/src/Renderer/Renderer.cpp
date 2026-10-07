/*
** EPITECH PROJECT, 2026
** zappy
** File description:
** Renderer
*/

#include "Renderer.hpp"
#include "Circle.hpp"
#include <chrono>
#include <string>
#include <vector>
#include <algorithm>

namespace zappy {

    void Renderer::setupColors() {
        _colors.setupColors();
    }

    void Renderer::createWindow(bool _isFullscreen) {
        _window.Init(1920, 1080, "Zappy GUI", 0, LOG_NONE);
        _window.SetTargetFPS(60);
        _window.SetFullscreen(_isFullscreen);
        _logo   = LoadImage("assets/images/logo.png");
        _cursor = LoadTexture("assets/images/cursor.png");
        _window.SetIcon(_logo);
        _font = raylib::Font("assets/font/akashi.ttf");
        _logo.Unload();
        _textBeginPosX = _window.GetWidth() / 2;
    }

    void Renderer::displayBasicGui(bool isFpsDisplayed, bool isCursorDefault) {
        if (isFpsDisplayed == true) {
            _window.DrawFPS(20, 20);
        }
        if (isCursorDefault == false) {
            _window.HideCursor();
            raylib::Vector2 pos = raylib::Mouse::GetPosition();
            _cursor.Draw(static_cast<int>(pos.x), static_cast<int>(pos.y), WHITE);
        } else {
            _window.ShowCursor();
        }
    }

    void Renderer::displayStats(const Server& server) {
        const int x = _textBeginPosX;
        const int y = _textBeginPosY;

        _font.DrawText("INFORMATIONS :", x, y, _fontSize + 10, _spacing, _colors.getTextColor());
        _font.DrawText("Players: " + std::to_string(server.players().getNbPlayers()), x, y + 40, _fontSize, _spacing, _colors.getTextColor());
        _font.DrawText("Deaths: " + std::to_string(server.players().getNbDeath()), x + 150, y + 40, _fontSize, _spacing, _colors.getTextColor());
        _font.DrawText("Frequency: " + std::to_string(server.world().getTimeReference()), x + 300, y + 40, _fontSize, _spacing, _colors.getTextColor());
        _font.DrawText("Map size: " + std::to_string(server.world().getMapSize().first) + "x" + std::to_string(server.world().getMapSize().second), x + 450, y + 40, _fontSize,
                       _spacing, _colors.getTextColor());
    }

    void Renderer::displayTotalResources(const Server& server) {
        const int x = _textBeginPosX;
        const int y = _textBeginPosY;

        _font.DrawText("TOTAL RESOURCES :", x, y + 80, _fontSize + 10, _spacing, _colors.getTextColor());
        _font.DrawText("Food: " + std::to_string(server.resources().getTotalFood()), x, y + 120, _fontSize, _spacing, _colors.getTextColor());
        _font.DrawText("Linemate: " + std::to_string(server.resources().getTotalLinemate()), x + 180, y + 120, _fontSize, _spacing, _colors.getTextColor());
        _font.DrawText("Deraumere: " + std::to_string(server.resources().getTotalDeraumere()), x + 370, y + 120, _fontSize, _spacing, _colors.getTextColor());
        _font.DrawText("Sibur: " + std::to_string(server.resources().getTotalSibur()), x + 570, y + 120, _fontSize, _spacing, _colors.getTextColor());
        _font.DrawText("Mendiane: " + std::to_string(server.resources().getTotalMendiane()), x + 730, y + 120, _fontSize, _spacing, _colors.getTextColor());
        _font.DrawText("Phiras: " + std::to_string(server.resources().getTotalPhiras()), x, y + 150, _fontSize, _spacing, _colors.getTextColor());
        _font.DrawText("Thystame: " + std::to_string(server.resources().getTotalThystame()), x + 160, y + 150, _fontSize, _spacing, _colors.getTextColor());
    }

    void Renderer::displaySelectedTile(const Server& server) {
        const int  x    = _textBeginPosX;
        const int  y    = _textBeginPosY;
        const auto tile = server.world().catchTile(_selectedTile.first, _selectedTile.second);
        if (!tile)
            return;
        _font.DrawText("SELECTED TILE : (x = " + std::to_string(_selectedTile.first) + " y = " + std::to_string(_selectedTile.second) + ") :", x, y + 200, _fontSize + 10, _spacing,
                       _colors.getTextColor());
        _font.DrawText("Food: " + std::to_string(tile->getRessources()[0]), x + 425, y + 205, _fontSize, _spacing, _colors.getTextColor());
        _font.DrawText("Linemate: " + std::to_string(tile->getRessources()[1]), x + 525, y + 205, _fontSize, _spacing, _colors.getTextColor());
        _font.DrawText("Deraumere: " + std::to_string(tile->getRessources()[2]), x + 650, y + 205, _fontSize, _spacing, _colors.getTextColor());
        _font.DrawText("Sibur: " + std::to_string(tile->getRessources()[3]), x + 800, y + 205, _fontSize, _spacing, _colors.getTextColor());
        _font.DrawText("Mendiane: " + std::to_string(tile->getRessources()[4]), x, y + 230, _fontSize, _spacing, _colors.getTextColor());
        _font.DrawText("Phiras: " + std::to_string(tile->getRessources()[5]), x + 150, y + 230, _fontSize, _spacing, _colors.getTextColor());
        _font.DrawText("Thystame: " + std::to_string(tile->getRessources()[6]), x + 300, y + 230, _fontSize, _spacing, _colors.getTextColor());
    }

    void Renderer::displaySelectedPlayer(const Server& server) {
        const int  x       = 1000;
        const int  y       = 200;
        const auto players = server.getPlayersSafe();
        if (!_selectedPlayerId.has_value()) {
            _font.DrawText("No player selected.", x + 200, 400, 20, _spacing, _colors.getTextColor());
            return;
        }
        auto it = std::find_if(players.begin(), players.end(), [&](const Player& player) { return player.getId() == _selectedPlayerId.value(); });
        if (it == players.end()) {
            _selectedPlayerId.reset();
            _font.DrawText("Selected player no longer exists.", x + 200, 400, 20, _spacing, _colors.getTextColor());
            return;
        }
        const Player& player = *it;
        _font.DrawText("SELECTED PLAYER : " + std::to_string(player.getId() + 1), x + 300, y + 200, _fontSize + 10, _spacing, _colors.getTextColor());
        _font.DrawText(std::string("Status : ") + (player.getIsAlive() ? "ALIVE" : "DEAD"), x + 300, y + 250, _fontSize, _spacing, _colors.getTextColor());
        if (!player.getIsAlive())
            return;
        _font.DrawText("Level : " + std::to_string(player.getLevel()), x + 300, y + 280, _fontSize, _spacing, _colors.getTextColor());
        _font.DrawText("Food : " + std::to_string(player.getFood()), x + 300, y + 310, _fontSize, _spacing, _colors.getTextColor());
        _font.DrawText("Linemate : " + std::to_string(player.getLinemate()), x + 300, y + 340, _fontSize, _spacing, _colors.getTextColor());
        _font.DrawText("Deraumere : " + std::to_string(player.getDeraumere()), x + 300, y + 370, _fontSize, _spacing, _colors.getTextColor());
        _font.DrawText("Sibur : " + std::to_string(player.getSibur()), x + 300, y + 400, _fontSize, _spacing, _colors.getTextColor());
        _font.DrawText("Mendiane : " + std::to_string(player.getMendiane()), x + 300, y + 430, _fontSize, _spacing, _colors.getTextColor());
        _font.DrawText("Phiras : " + std::to_string(player.getPhiras()), x + 300, y + 460, _fontSize, _spacing, _colors.getTextColor());
        _font.DrawText("Thystame : " + std::to_string(player.getThystame()), x + 300, y + 490, _fontSize, _spacing, _colors.getTextColor());
        _font.DrawText("Team : " + player.getTeam(), x + 300, y + 520, _fontSize, _spacing, _colors.getTextColor());
    }

    void Renderer::displayPlayers(Server& server, int offsetX, int offsetY, int tileSizeClamped) {
        std::vector<Player> players = server.getPlayersSafe();

        {
            std::lock_guard<std::mutex> lock(server.getMutex());
            auto&                       realPlayers = server.players().getPlayer();
            for (auto& player : realPlayers) {
                if (player.getId() < 0)
                    player.setId();
            }
        }
        int id = 1;
        for (auto& player : players) {
            if (!player.getIsAlive()) {
                id++;
                continue;
            }
            const int x = player.getPos().first.first;
            const int y = player.getPos().first.second;
            if (x < 0 || y < 0)
                continue;
            const int      playerX = offsetX + x * tileSizeClamped + tileSizeClamped / 2;
            const int      playerY = offsetY + y * tileSizeClamped + tileSizeClamped / 2;
            raylib::Circle circle;
            circle.CircleDraw(playerX, playerY, std::max(4, tileSizeClamped / 6), _colors.getPlayerColor());
            _font.DrawText("P" + std::to_string(id), playerX - 6, playerY - 8, 12, 1.0f, _colors.getPlayerNumberColor());
            id++;
        }
    }

    void Renderer::updateTiles(const Server& server) {
        const std::pair<size_t, size_t> mapSize        = server.world().getMapSize();
        const int                       mapWidth       = mapSize.first;
        const int                       mapHeight      = mapSize.second;
        const int                       hudHeight      = 100;
        const int                       padding        = 10;
        const int                       availableWidth = (_window.GetWidth() / 2) - 2 * 10;
        const int                       offsetX        = 10;
        const int                       offsetY        = 100 + padding;
        const std::vector<Tile>         tiles          = server.getTilesSafe();
        const Tile                      fallbackTile{};
        if (mapWidth <= 0)
            return;
        const int tileSize        = std::min(availableWidth / mapWidth, (_window.GetHeight() - hudHeight - 2 * padding) / mapHeight);
        const int tileSizeClamped = tileSize;

        _drawnTiles.clear();
        _tileResources.clear();
        for (int y = 0; y < mapHeight; ++y) {
            std::vector<raylib::Rectangle> line;
            std::vector<std::vector<int>>  resLine;
            for (int x = 0; x < mapWidth; ++x) {
                const int        rectX     = offsetX + x * tileSizeClamped;
                const int        rectY     = offsetY + y * tileSizeClamped;
                const int        tileIndex = y * mapWidth + x;
                const Tile&      tile      = (tileIndex < static_cast<int>(tiles.size())) ? tiles[tileIndex] : fallbackTile;
                std::vector<int> resources = tile.getRessources();
                resources.resize(7, 0);
                line.emplace_back(rectX, rectY, tileSizeClamped - 1, tileSizeClamped - 1);
                resLine.push_back(resources);
            }
            _drawnTiles.push_back(line);
            _tileResources.push_back(resLine);
        }
    }

    void Renderer::drawTiles() {
        static const ::Color resourceColors[7] = {RED, GREEN, BLUE, YELLOW, PURPLE, ORANGE, PINK};
        // dot layout: 2 columns, 4 rows inside tile
        static const float dotColRatio[7] = {0.25f, 0.75f, 0.25f, 0.75f, 0.25f, 0.75f, 0.5f};
        static const float dotRowRatio[7] = {0.2f, 0.2f, 0.4f, 0.4f, 0.6f, 0.6f, 0.8f};

        for (std::size_t y = 0; y < _drawnTiles.size(); ++y) {
            for (std::size_t x = 0; x < _drawnTiles[y].size(); ++x) {
                const auto& rect = _drawnTiles[y][x];
                rect.Draw(_colors.getTileColor());
                rect.DrawLines(_colors.getTileBorderColor());

                if (y >= _tileResources.size() || x >= _tileResources[y].size())
                    continue;
                const auto& res    = _tileResources[y][x];
                const int   tileW  = static_cast<int>(rect.width);
                const int   tileH  = static_cast<int>(rect.height);
                const int   radius = std::max(2, std::min(tileW, tileH) / 8);
                for (int r = 0; r < 7; ++r) {
                    if (r >= static_cast<int>(res.size()) || res[r] == 0)
                        continue;
                    const int cx = static_cast<int>(rect.x) + static_cast<int>(dotColRatio[r] * tileW);
                    const int cy = static_cast<int>(rect.y) + static_cast<int>(dotRowRatio[r] * tileH);
                    DrawCircle(cx, cy, radius, resourceColors[r]);
                }
            }
        }
    }

    void Renderer::displayTiles(const Server& server) {
        updateTiles(server);
        drawTiles();
    }

    void Renderer::updateBroadcasters(std::vector<int> vec, Server& server) {
        const int                       offsetX        = 10;
        const int                       offsetY        = 110;
        const int                       availableWidth = (_window.GetWidth() / 2) - 2 * 10;
        const std::pair<size_t, size_t> mapSize        = server.world().getMapSize();
        const int                       tileSize       = std::min(availableWidth / (int)mapSize.first, (_window.GetHeight() - 120) / (int)mapSize.second);
        auto&                           allPlayers     = server.players().getPlayer();

        for (int i = 0; i < (int)vec.size(); i++) {
            if (vec[i] >= 0 && static_cast<size_t>(vec[i]) < allPlayers.size()) {
                Player&   player  = allPlayers[vec[i]];
                const int x       = player.getPos().first.first;
                const int y       = player.getPos().first.second;
                const int playerX = offsetX + x * tileSize + tileSize / 2;
                const int playerY = offsetY + y * tileSize + tileSize / 2;
                if (player.getIsAlive() == true)
                    _broadcasters.push_back({playerX, playerY, std::chrono::high_resolution_clock::now()});
            }
        }
    }

    void Renderer::updateIncreasers(std::vector<int> vec, Server& server) {
        const int                       offsetX        = 10;
        const int                       offsetY        = 110;
        const int                       availableWidth = (_window.GetWidth() / 2) - 2 * 10;
        const std::pair<size_t, size_t> mapSize        = server.world().getMapSize();
        const int                       tileSize       = std::min(availableWidth / (int)mapSize.first, (_window.GetHeight() - 120) / (int)mapSize.second);
        auto&                           players        = server.players().getPlayer();
        for (int id : vec) {
            if (id < 0 || static_cast<size_t>(id) >= players.size())
                continue;
            Player& player = players[id];
            if (!player.getIsAlive())
                continue;
            const int x       = player.getPos().first.first;
            const int y       = player.getPos().first.second;
            const int playerX = offsetX + x * tileSize + tileSize / 2;
            const int playerY = offsetY + y * tileSize + tileSize / 2;
            if (player.getIsAlive() == true)
                _increasers.push_back({playerX, playerY, std::chrono::high_resolution_clock::now()});
        }
    }

    void Renderer::drawBroadcastCircles() {
        const float MAX_LIFETIME_S = 5.0f;
        const float MAX_RADIUS     = 80.0f;
        auto        now            = std::chrono::high_resolution_clock::now();

        _broadcasters.erase(std::remove_if(_broadcasters.begin(), _broadcasters.end(),
                                           [&](const BroadcastCircle& c) {
                                               float elapsed = std::chrono::duration<float>(now - c.spawnTime).count();
                                               return elapsed >= MAX_LIFETIME_S;
                                           }),
                            _broadcasters.end());
        BeginBlendMode(BLEND_ALPHA);
        for (const BroadcastCircle& c : _broadcasters) {
            float         elapsed = std::chrono::duration<float>(now - c.spawnTime).count();
            float         t       = elapsed / MAX_LIFETIME_S;
            float         radius  = t * MAX_RADIUS;
            unsigned char alpha   = static_cast<unsigned char>((1.0f - t) * 255);
            Color         color   = {255, 200, 50, alpha};
            raylib::Circle().DrawBorderCircle({(float)c.centerX, (float)c.centerY}, radius - 2.0f, radius + 2.0f, 0.0f, 360.0f, 36, color);
        }
        EndBlendMode();
    }

    void Renderer::drawIncreaseCircles() {
        const float MAX_LIFETIME_S = 5.0f;
        const float MAX_RADIUS     = 80.0f;
        auto        now            = std::chrono::high_resolution_clock::now();

        _increasers.erase(std::remove_if(_increasers.begin(), _increasers.end(),
                                         [&](const BroadcastCircle& c) {
                                             float elapsed = std::chrono::duration<float>(now - c.spawnTime).count();
                                             return elapsed >= MAX_LIFETIME_S;
                                         }),
                          _increasers.end());
        BeginBlendMode(BLEND_ALPHA);
        for (const BroadcastCircle& c : _increasers) {
            float         elapsed = std::chrono::duration<float>(now - c.spawnTime).count();
            float         t       = elapsed / MAX_LIFETIME_S;
            float         radius  = t * MAX_RADIUS;
            unsigned char alpha   = static_cast<unsigned char>((1.0f - t) * 255);
            Color         color   = {0, 255, 0, alpha};
            raylib::Circle().DrawBorderCircle({(float)c.centerX, (float)c.centerY}, radius - 2.0f, radius + 2.0f, 0.0f, 360.0f, 36, color);
        }
        EndBlendMode();
    }

    void Renderer::displayLastLogs(Server& server) {
        server.trashLogs(15);
        const std::vector<std::string> logs = server.logs();
        int                            x    = 960;
        int                            y    = 400;
        _font.DrawText("LOGS :", x, y, 25, 1.0f, _colors.getTextColor());
        y += 35;
        for (const std::string& log : logs) {
            _font.DrawText(log, x, y, 12, 1.0f, _colors.getTextColor());
            y += 22;
        }
    }

    void Renderer::displayServerElements(Server& server) {
        _font.DrawText("ZAPFRITES", (_window.GetSize().GetX() / 2) - 3 * 50, 20, 50, 5, _colors.getTextColor());
        displayStats(server);
        displayLastLogs(server);
        displayTotalResources(server);
        displaySelectedTile(server);
        displaySelectedPlayer(server);
        const std::pair<size_t, size_t> mapSize = server.world().getMapSize();
        if (mapSize.first == 0 || mapSize.second == 0) {
            _font.DrawText("Waiting for map data...", 10, 150, 20, 1.0f, _colors.getTextColor());
            return;
        }
        const int availableWidth = (_window.GetWidth() / 2) - 2 * 10;
        const int tileSize       = std::min(availableWidth / mapSize.first, (_window.GetHeight() - 120) / mapSize.second);
        displayTiles(server);
        displayPlayers(server, 10, 110, tileSize);
    }

    void Renderer::displayAll(Server& server, bool isFpsDisplayed, bool isCursorDefault) {
        _window.BeginDrawing();
        _window.ClearBackground(_colors.getBackgroundColor());
        displayServerElements(server);
        displayBasicGui(isFpsDisplayed, isCursorDefault);
        drawBroadcastCircles();
        drawIncreaseCircles();
        const auto& winner = server.world().getWinner();
        if (winner.has_value())
            displayWinner(winner.value());
        _window.EndDrawing();
    }

    void Renderer::displayWinner(const std::string& team) {
        const int w = _window.GetWidth();
        const int h = _window.GetHeight();
        DrawRectangle(0, 0, w, h, Color{0, 0, 0, 200});
        const std::string line1 = "TEAM " + team;
        const std::string line2 = "WINS!";
        Vector2           s1    = MeasureTextEx(_font, line1.c_str(), 80, _spacing);
        Vector2           s2    = MeasureTextEx(_font, line2.c_str(), 100, _spacing);
        _font.DrawText(line1, (int)((w - s1.x) / 2), (int)(h / 2.0f - s1.y - 10), 80, _spacing, GOLD);
        _font.DrawText(line2, (int)((w - s2.x) / 2), (int)(h / 2.0f) + 10, 100, _spacing, GOLD);
    }

    void Renderer::close() {
        _cursor.Unload();
        _font.Unload();
        _window.Close();
    }

    raylib::Window& Renderer::getWindow() {
        return _window;
    }

    Colors& Renderer::getColors() {
        return _colors;
    }

    void Renderer::setSelectedTile(std::pair<int, int>& pair) {
        _selectedTile = pair;
    }

    void Renderer::setSelectedPlayerId(std::optional<int> i) {
        _selectedPlayerId = i;
    }

    std::vector<std::vector<raylib::Rectangle>> Renderer::getDrawnTiles() {
        return _drawnTiles;
    }
    std::optional<int> Renderer::getSelectedPlayerId() {
        return _selectedPlayerId;
    }
}
