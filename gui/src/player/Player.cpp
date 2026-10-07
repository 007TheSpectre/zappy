/*
** EPITECH PROJECT, 2026
** zappy
** File description:
** Player
*/

#include "Player.hpp"
#include <cstdio>
#include <string>
#include <vector>

namespace zappy {

    std::string orientationToString(Orientation orientation) {
        switch (orientation) {
            case NORTH: return "NORTH";
            case EAST: return "EAST";
            case SOUTH: return "SOUTH";
            case WEST: return "WEST";
            default: return "UNKNOWN";
        }
    }

    u_int8_t Player::getLevel() const {
        return _lvl;
    }

    size_t Player::getFood() const {
        return _totalFood;
    }

    size_t Player::getLinemate() const {
        return _totalLinemate;
    }

    size_t Player::getDeraumere() const {
        return _totalDeraumere;
    }

    size_t Player::getSibur() const {
        return _totalSibur;
    }

    size_t Player::getMendiane() const {
        return _totalMendiane;
    }

    size_t Player::getPhiras() const {
        return _totalPhiras;
    }

    size_t Player::getThystame() const {
        return _totalThystame;
    }

    bool Player::getIsAlive() const {
        return _isAlive;
    }

    std::pair<std::pair<int, int>, Orientation> Player::getPos() const {
        return _pos;
    }

    int Player::getId() const {
        return _id;
    }
    void Player::setLevel(u_int8_t lvl) {
        _lvl = lvl;
    }

    void Player::setIsAlive(bool alive) {
        _isAlive = alive;
    }

    void Player::setPos(std::pair<std::pair<int, int>, Orientation> pos) {
        _pos = pos;
    }

    void Player::setId() {
        static int id = 0;
        _id           = id;
        id++;
    }

    std::string Player::getTeam() const {
        return _team;
    }

    void Player::setTeam(std::string& name) {
        _team = name;
    }

    void Player::setInventory(const std::vector<int>& res) {
        if (res.size() < 7)
            return;
        _totalFood      = static_cast<size_t>(res[0]);
        _totalLinemate  = static_cast<size_t>(res[1]);
        _totalDeraumere = static_cast<size_t>(res[2]);
        _totalSibur     = static_cast<size_t>(res[3]);
        _totalMendiane  = static_cast<size_t>(res[4]);
        _totalPhiras    = static_cast<size_t>(res[5]);
        _totalThystame  = static_cast<size_t>(res[6]);
    }
}
