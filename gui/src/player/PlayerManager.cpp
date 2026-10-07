/*
** EPITECH PROJECT, 2026
** zappy
** File description:
** PlayerManager
*/

#include "PlayerManager.hpp"
#include <cstddef>
#include <vector>

namespace zappy {

    std::vector<Player>& PlayerManager::getPlayer() {
        return _Players;
    }

    const std::vector<Player>& PlayerManager::getPlayer() const {
        return _Players;
    }

    size_t PlayerManager::getNbPlayers() const {
        return _nbPlayers;
    }

    size_t PlayerManager::getNbDeath() const {
        return _nbDeath;
    }

    void PlayerManager::getAllAlive() {
        int nbAlive  = 0;
        int nbDeaths = 0;
        for (int i = 0; i < _Players.size(); i++) {
            if (_Players[i].getIsAlive())
                nbAlive++;
            else {
                nbDeaths++;
            }
        }
        _nbPlayers = nbAlive;
        _nbDeath   = nbDeaths;
    }
    std::vector<int> PlayerManager::takeBroadcasters() {
        return std::move(_broadcasters);
    }

    void PlayerManager::clearBroadcasters() {
        _broadcasters.clear();
    }

    void PlayerManager::addBroadcaster(int i) {
        _broadcasters.push_back(i);
    }

    std::vector<int> PlayerManager::takeIncreasers() {
        return std::move(_lvlIncreasers);
    }

    void PlayerManager::clearIncreasers() {
        _lvlIncreasers.clear();
    }

    void PlayerManager::addIncreaser(int i) {
        _lvlIncreasers.push_back(i);
    }

}
