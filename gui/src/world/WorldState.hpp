/*
** EPITECH PROJECT, 2026
** zappy
** File description:
** WorldState
*/

#pragma once

#include <optional>
#include <string>
#include <sys/types.h>
#include <utility>
#include <vector>
#include "Tile.hpp"

namespace zappy {
    class WorldState {
      public:
        std::pair<size_t, size_t>         getMapSize() const;
        const std::vector<Tile>&          getTiles() const;
        std::optional<const Tile>         catchTile(int x, int y) const;
        int                               getTimeReference() const;

        void                              setMapSize(int x, int y);
        void                              setTimeReference(int i);
        void                              setTiles(std::vector<Tile> tiles);
        void                              updateTile(int x, int y, std::vector<int> resources);
        void                              setWinner(const std::string& team);
        const std::optional<std::string>& getWinner() const;

      private:
        std::pair<size_t, size_t>  _mapSize;
        std::vector<Tile>          _tiles;
        int                        _timeReference = 0;
        std::optional<std::string> _winner;
    };
}