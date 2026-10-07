/*
** EPITECH PROJECT, 2026
** zappy
** File description:
** ResourceTracker
*/

#pragma once

#include <sys/types.h>
#include <vector>
#include "../world/Tile.hpp"
namespace zappy {
    class ResourceTracker {
      public:
        size_t getTotalFood() const;
        size_t getTotalLinemate() const;
        size_t getTotalDeraumere() const;
        size_t getTotalSibur() const;
        size_t getTotalMendiane() const;
        size_t getTotalPhiras() const;
        size_t getTotalThystame() const;

        void   incrementFood(int i);
        void   resetFood(int i);
        void   incrementLinemate(int i);
        void   resetLinemate(int i);
        void   incrementDeraumere(int i);
        void   resetDeraumere(int i);
        void   incrementSibur(int i);
        void   resetSibur(int i);
        void   incrementMendiane(int i);
        void   resetMendiane(int i);
        void   incrementPhiras(int i);
        void   resetPhiras(int i);
        void   incrementThystame(int i);
        void   resetThystame(int i);

        void   updateResources(std::vector<Tile> tiles);

      private:
        size_t _totalFood      = 0;
        size_t _totalLinemate  = 0;
        size_t _totalDeraumere = 0;
        size_t _totalSibur     = 0;
        size_t _totalMendiane  = 0;
        size_t _totalPhiras    = 0;
        size_t _totalThystame  = 0;
    };
}