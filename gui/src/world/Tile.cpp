/*
** EPITECH PROJECT, 2026
** zappy
** File description:
** Tile
*/

#include "Tile.hpp"

namespace zappy {

    std::vector<int> Tile::getRessources() const {
        return _ressources;
    }

    std::pair<int, int> Tile::getPosition() const {
        return _position;
    }

    void Tile::setPosition(int x, int y) {
        _position.first  = x;
        _position.second = y;
    }
    void Tile::setRessources(std::vector<int> vec) {
        _ressources.clear();
        _ressources = vec;
    }

}