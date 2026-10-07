/*
** EPITECH PROJECT, 2026
** zappy
** File description:
** Colors
*/

#pragma once

#include <raylib-cpp.hpp>
#include <vector>

namespace zappy {

    class Colors {
      public:
        void                              setupColors();
        void                              handleEvents();

        const std::vector<raylib::Color>& getRessourceColors() const {
            return _ressourceColors;
        }
        raylib::Color getTileColor() const;
        raylib::Color getTileBorderColor() const;
        raylib::Color getBackgroundColor() const;
        raylib::Color getTextColor() const;
        raylib::Color getPlayerColor() const;
        raylib::Color getPlayerNumberColor() const;
        size_t        getPlayerColorIndex() const;
        size_t        getPlayerNumberColorIndex() const;
        size_t        getTileColorIndex() const;
        size_t        getTileBorderColorIndex() const;
        size_t        getBackgroundColorIndex() const;
        size_t        getTextColorIndex() const;

      private:
        std::vector<raylib::Color> _ressourceColors;
        raylib::Color              _TileColor;
        raylib::Color              _TileBorderColor;
        raylib::Color              _BackgroundColor;
        raylib::Color              _TextColor;
        raylib::Color              _PlayerColor;
        raylib::Color              _PlayerNumberColor;
        size_t                     _PlayerColorIndex;
        size_t                     _PlayerNumberColorIndex;
        size_t                     _tileColorIndex;
        size_t                     _tileBorderColorIndex;
        size_t                     _backgroundColorIndex;
        size_t                     _TextColorIndex;
    };
}