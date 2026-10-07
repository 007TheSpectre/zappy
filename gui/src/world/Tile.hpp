/*
** EPITECH PROJECT, 2026
** zappy
** File description:
** Map
*/

#pragma once

#include <vector>

namespace zappy {
    class Tile {
      public:
        std::vector<int>    getRessources() const;
        std::pair<int, int> getPosition() const;
        void                setPosition(int x, int y);
        void                setRessources(std::vector<int> vec);

      private:
        std::vector<int>    _ressources;
        std::pair<int, int> _position;
    };
}