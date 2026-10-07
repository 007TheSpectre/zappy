/*
** EPITECH PROJECT, 2026
** zappy
** File description:
** ResourceTracker
*/

#include "ResourceTracker.hpp"

namespace zappy {
    void ResourceTracker::incrementFood(int i) {
        _totalFood += static_cast<size_t>(i);
    }

    void ResourceTracker::incrementLinemate(int i) {
        _totalLinemate += static_cast<size_t>(i);
    }

    void ResourceTracker::incrementDeraumere(int i) {
        _totalDeraumere += static_cast<size_t>(i);
    }

    void ResourceTracker::incrementSibur(int i) {
        _totalSibur += static_cast<size_t>(i);
    }

    void ResourceTracker::incrementMendiane(int i) {
        _totalMendiane += static_cast<size_t>(i);
    }

    void ResourceTracker::incrementPhiras(int i) {
        _totalPhiras += static_cast<size_t>(i);
    }

    void ResourceTracker::incrementThystame(int i) {
        _totalThystame += static_cast<size_t>(i);
    }

    void ResourceTracker::resetFood(int i) {
        _totalFood = i;
    }

    void ResourceTracker::resetLinemate(int i) {
        _totalLinemate = i;
    }

    void ResourceTracker::resetDeraumere(int i) {
        _totalDeraumere = i;
    }

    void ResourceTracker::resetSibur(int i) {
        _totalSibur = i;
    }

    void ResourceTracker::resetMendiane(int i) {
        _totalMendiane = i;
    }

    void ResourceTracker::resetPhiras(int i) {
        _totalPhiras = i;
    }

    void ResourceTracker::resetThystame(int i) {
        _totalThystame = i;
    }
    size_t ResourceTracker::getTotalFood() const {
        return _totalFood;
    }

    size_t ResourceTracker::getTotalLinemate() const {
        return _totalLinemate;
    }

    size_t ResourceTracker::getTotalDeraumere() const {
        return _totalDeraumere;
    }

    size_t ResourceTracker::getTotalSibur() const {
        return _totalSibur;
    }
    size_t ResourceTracker::getTotalMendiane() const {
        return _totalMendiane;
    }
    size_t ResourceTracker::getTotalPhiras() const {
        return _totalPhiras;
    }
    size_t ResourceTracker::getTotalThystame() const {
        return _totalThystame;
    }

    void ResourceTracker::updateResources(std::vector<Tile> tiles) {
        resetFood(0);
        resetDeraumere(0);
        resetLinemate(0);
        resetMendiane(0);
        resetPhiras(0);
        resetSibur(0);
        resetThystame(0);

        for (size_t i = 0; i < tiles.size(); ++i) {
            const auto& resources = tiles[i].getRessources();
            for (size_t j = 0; j < resources.size() && j < 7; ++j) {
                switch (j) {
                    case 0: incrementFood(resources[j]); break;
                    case 1: incrementLinemate(resources[j]); break;
                    case 2: incrementDeraumere(resources[j]); break;
                    case 3: incrementSibur(resources[j]); break;
                    case 4: incrementMendiane(resources[j]); break;
                    case 5: incrementPhiras(resources[j]); break;
                    case 6: incrementThystame(resources[j]); break;
                    default: break;
                }
            }
            tiles[i].setRessources(resources);
        }
    }
}