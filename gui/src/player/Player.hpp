/*
** EPITECH PROJECT, 2026
** zappy
** File description:
** Player
*/

#pragma once
#include <cstddef>
#include <string>
#include <sys/types.h>
#include <utility>
#include <vector>

namespace zappy {
    enum Orientation {
        NORTH = 1,
        EAST  = 2,
        SOUTH = 3,
        WEST  = 4
    };

    std::string orientationToString(Orientation orientation);

    class Player {
      public:
        u_int8_t                                    getLevel() const;
        size_t                                      getFood() const;
        size_t                                      getLinemate() const;
        size_t                                      getDeraumere() const;
        size_t                                      getSibur() const;
        size_t                                      getMendiane() const;
        size_t                                      getPhiras() const;
        size_t                                      getThystame() const;
        bool                                        getIsAlive() const;
        std::pair<std::pair<int, int>, Orientation> getPos() const;
        int                                         getId() const;
        std::string                                 getTeam() const;
        void                                        setLevel(u_int8_t lvl);
        void                                        setIsAlive(bool alive);
        void                                        setPos(std::pair<std::pair<int, int>, Orientation> pos);
        void                                        setId();
        void                                        setTeam(std::string& name);
        // Sets food + the six craftable resources from a parsed `pin` payload.
        // res = {food, linemate, deraumere, sibur, mendiane, phiras, thystame}.
        void setInventory(const std::vector<int>& res);

      private:
        u_int8_t                                    _lvl            = 1;
        int                                         _id             = -1;
        size_t                                      _totalFood      = 0;
        size_t                                      _totalLinemate  = 0;
        size_t                                      _totalDeraumere = 0;
        size_t                                      _totalSibur     = 0;
        size_t                                      _totalMendiane  = 0;
        size_t                                      _totalPhiras    = 0;
        size_t                                      _totalThystame  = 0;
        bool                                        _isAlive        = true;
        std::string                                 _team           = "unk";
        std::pair<std::pair<int, int>, Orientation> _pos;
    };

}
