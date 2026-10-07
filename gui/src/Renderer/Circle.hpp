/*
** EPITECH PROJECT, 2026
** zappy
** File description:
** Circle
*/

#pragma once
#include <raylib-cpp.hpp>
#include <raylib.h>

namespace raylib {
    class Circle {
      public:
        void CircleDraw(int centerX, int centerY, float radius, Color color) {
            return DrawCircle(centerX, centerY, radius, color);
        }
        void DrawBorderCircle(Vector2 center, float innerRadius, float outerRadius, float startAngle, float endAngle, int segments, Color color) {
            return DrawRing(center, innerRadius, outerRadius, startAngle, endAngle, segments, color);
        }
    };
}