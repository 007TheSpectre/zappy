/*
** EPITECH PROJECT, 2026
** zappy
** File description:
** PlayerManager
*/

#pragma once

#include <vector>
#include "Player.hpp"

namespace zappy {
    class PlayerManager {
      public:
        std::vector<Player>&       getPlayer();
        const std::vector<Player>& getPlayer() const;
        size_t                     getNbPlayers() const;
        size_t                     getNbDeath() const;
        void                       getAllAlive();
        std::vector<int>           takeBroadcasters();
        void                       clearBroadcasters();
        void                       addBroadcaster(int i);
        std::vector<int>           takeIncreasers();
        void                       clearIncreasers();
        void                       addIncreaser(int i);

      private:
        std::vector<Player> _Players;
        std::vector<int>    _broadcasters;
        std::vector<int>    _lvlIncreasers;
        size_t              _nbPlayers = 0;
        size_t              _nbDeath   = 0;
    };
}