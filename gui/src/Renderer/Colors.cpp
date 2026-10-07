/*
** EPITECH PROJECT, 2026
** zappy
** File description:
** Colors
*/

#include "Colors.hpp"
namespace zappy {
    void Colors::setupColors() {
        _ressourceColors                        = {RED, GREEN, BLUE, YELLOW, PURPLE, ORANGE, PINK, BROWN, GRAY, LIGHTGRAY, MAROON, LIME, SKYBLUE, VIOLET, BEIGE, DARKBROWN, GOLD};
        std::vector<raylib::Color> secondColors = {DARKGREEN, DARKBLUE, DARKPURPLE, DARKBROWN, BLACK, WHITE};
        _ressourceColors.insert(_ressourceColors.end(), secondColors.begin(), secondColors.end());
        _TileColor              = GRAY;
        _tileColorIndex         = 8;
        _TileBorderColor        = BLACK;
        _tileBorderColorIndex   = 21;
        _TextColor              = WHITE;
        _TextColorIndex         = 22;
        _BackgroundColor        = BLACK;
        _backgroundColorIndex   = 21;
        _PlayerColor            = RED;
        _PlayerColorIndex       = 0;
        _PlayerNumberColor      = BLACK;
        _PlayerNumberColorIndex = 21;
    }

    void Colors::handleEvents() {
        if (raylib::Keyboard::IsKeyPressed(KEY_ONE)) {
            _tileColorIndex = (_tileColorIndex + 1) % _ressourceColors.size();
            while (_TextColorIndex == _tileColorIndex || _backgroundColorIndex == _tileColorIndex || _tileColorIndex == 0)
                _tileColorIndex = (_tileColorIndex + 1) % _ressourceColors.size();
            _TileColor = _ressourceColors[_tileColorIndex];
        }
        if (raylib::Keyboard::IsKeyPressed(KEY_TWO)) {
            _tileBorderColorIndex = (_tileBorderColorIndex + 1) % _ressourceColors.size();
            _TileBorderColor      = _ressourceColors[_tileBorderColorIndex];
        }
        if (raylib::Keyboard::IsKeyPressed(KEY_THREE)) {
            _backgroundColorIndex = (_backgroundColorIndex + 1) % _ressourceColors.size();
            while (_backgroundColorIndex == _tileColorIndex || _backgroundColorIndex == _TextColorIndex)
                _backgroundColorIndex = (_backgroundColorIndex + 1) % _ressourceColors.size();
            _BackgroundColor = _ressourceColors[_backgroundColorIndex];
        }
        if (raylib::Keyboard::IsKeyPressed(KEY_FOUR)) {
            _TextColorIndex = (_TextColorIndex + 1) % _ressourceColors.size();
            while (_TextColorIndex == _backgroundColorIndex || _TextColorIndex == _tileColorIndex)
                _TextColorIndex = (_TextColorIndex + 1) % _ressourceColors.size();
            _TextColor = _ressourceColors[_TextColorIndex];
        }
        if (raylib::Keyboard::IsKeyPressed(KEY_FIVE)) {
            _PlayerColorIndex = (_PlayerColorIndex + 1) % _ressourceColors.size();
            while (_PlayerColorIndex == _tileColorIndex || _PlayerColorIndex == _PlayerNumberColorIndex || _PlayerColorIndex == _TextColorIndex)
                _PlayerColorIndex = (_PlayerColorIndex + 1) % _ressourceColors.size();
            _PlayerColor = _ressourceColors[_PlayerColorIndex];
        }
        if (raylib::Keyboard::IsKeyPressed(KEY_SIX)) {
            _PlayerNumberColorIndex = (_PlayerNumberColorIndex + 1) % _ressourceColors.size();
            while (_PlayerNumberColorIndex == _tileColorIndex)
                _PlayerNumberColorIndex = (_PlayerNumberColorIndex + 1) % _ressourceColors.size();
            _PlayerNumberColor = _ressourceColors[_PlayerNumberColorIndex];
        }
    }

    size_t Colors::getTextColorIndex() const {
        return _TextColorIndex;
    }

    raylib::Color Colors::getTileColor() const {
        return _TileColor;
    }
    raylib::Color Colors::getTileBorderColor() const {
        return _TileBorderColor;
    }
    raylib::Color Colors::getBackgroundColor() const {
        return _BackgroundColor;
    }
    raylib::Color Colors::getTextColor() const {
        return _TextColor;
    }
    raylib::Color Colors::getPlayerColor() const {
        return _PlayerColor;
    }
    raylib::Color Colors::getPlayerNumberColor() const {
        return _PlayerNumberColor;
    }
    size_t Colors::getPlayerColorIndex() const {
        return _PlayerColorIndex;
    }
    size_t Colors::getPlayerNumberColorIndex() const {
        return _PlayerNumberColorIndex;
    }
    size_t Colors::getTileColorIndex() const {
        return _tileColorIndex;
    }
    size_t Colors::getTileBorderColorIndex() const {
        return _tileBorderColorIndex;
    }
    size_t Colors::getBackgroundColorIndex() const {
        return _backgroundColorIndex;
    }
}