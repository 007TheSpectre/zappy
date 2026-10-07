/*
** EPITECH PROJECT, 2026
** zappy
** File description:
** Renderer
*/

#pragma once

#include "../server/Server.hpp"
#include "Colors.hpp"
#include <optional>
#include <raylib-cpp.hpp>
#include <vector>
#include "BroadcastCircle.hpp"
namespace zappy {

    class Renderer {
      public:
        void                                        createWindow(bool _isFullscreen);
        void                                        displayTiles(const Server& server);
        void                                        drawTiles();
        void                                        updateTiles(const Server& server);
        void                                        displayBasicGui(bool isFpsDisplayed, bool isCursorDefault);
        void                                        displayServerElements(Server& server);
        void                                        displayPlayers(Server& server, int offsetX, int offsetY, int tileSizeClamped);
        void                                        setupColors();
        void                                        displayStats(const Server& server);
        void                                        displayLastLogs(Server& server);
        void                                        displaySelectedTile(const Server& server);
        void                                        displaySelectedPlayer(const Server& server);
        void                                        displayTotalResources(const Server& server);
        void                                        displayAll(Server& server, bool isFpsDisplayed, bool isCursorDefault);
        void                                        displayWinner(const std::string& team);
        void                                        close(void);
        raylib::Window&                             getWindow();
        Colors&                                     getColors();
        std::vector<std::vector<raylib::Rectangle>> getDrawnTiles();
        std::optional<int>                          getSelectedPlayerId();
        void                                        setSelectedTile(std::pair<int, int>& pair);
        void                                        updateBroadcasters(std::vector<int> vec, Server& server);
        void                                        updateIncreasers(std::vector<int> vec, Server& server);
        void                                        drawBroadcastCircles();
        void                                        drawIncreaseCircles();
        void                                        setSelectedPlayerId(std::optional<int> i);

      private:
        raylib::Image                               _logo;
        raylib::Font                                _font;
        raylib::Texture                             _cursor;
        raylib::Window                              _window;
        std::vector<std::vector<raylib::Rectangle>> _drawnTiles;
        std::vector<std::vector<std::vector<int>>>  _tileResources;
        std::pair<int, int>                         _selectedTile;
        std::optional<int>                          _selectedPlayerId;
        Colors                                      _colors;
        int                                         _textBeginPosX;
        const int                                   _textBeginPosY = 110;
        const int                                   _fontSize      = 15;
        const float                                 _spacing       = 2.0f;
        std::vector<BroadcastCircle>                _increasers;
        std::vector<BroadcastCircle>                _broadcasters;
    };
}
