/*
** EPITECH PROJECT, 2026
** zappy
** File description:
** Worldstate
*/

#include "WorldState.hpp"
#include <utility>

namespace zappy {
    std::pair<size_t, size_t> WorldState::getMapSize() const {
        return _mapSize;
    }
    const std::vector<Tile>& WorldState::getTiles() const {
        return _tiles;
    }
    int WorldState::getTimeReference() const {
        return _timeReference;
    }
    void WorldState::setMapSize(int x, int y) {
        _mapSize.first  = x;
        _mapSize.second = y;
    }

    void WorldState::setTimeReference(int i) {
        _timeReference = i;
    }
    void WorldState::setTiles(std::vector<Tile> tiles) {
        _tiles = std::move(tiles);
    }
    void WorldState::updateTile(int x, int y, std::vector<int> resources) {
        for (auto& tile : _tiles) {
            if (tile.getPosition().first == x && tile.getPosition().second == y) {
                tile.setRessources(resources);
                return;
            }
        }
        // First time we hear about this tile (e.g. the initial mct dump streamed
        // in line by line): create it so the map fills in instead of staying empty.
        Tile tile;
        tile.setPosition(x, y);
        tile.setRessources(resources);
        _tiles.push_back(tile);
    }

    void WorldState::setWinner(const std::string& team) {
        _winner = team;
    }

    const std::optional<std::string>& WorldState::getWinner() const {
        return _winner;
    }

    std::optional<const Tile> WorldState::catchTile(int x, int y) const {
        for (int i = 0; i < _tiles.size(); i++) {
            if (_tiles[i].getPosition().first == x && _tiles[i].getPosition().second == y)
                return _tiles[i];
        }
        return std::nullopt;
    }
}