/*
** EPITECH PROJECT, 2026
** zappy
** File description:
** BroadcastCircle
*/

#pragma once
#include <chrono>

class BroadcastCircle {
  public:
    int                                                         centerX;
    int                                                         centerY;
    std::chrono::time_point<std::chrono::high_resolution_clock> spawnTime;
};
