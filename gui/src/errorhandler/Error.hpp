/*
** EPITECH PROJECT, 2026
** Error
** File description:
** Throw exceptions
*/

#pragma once

#include <string>
#include <stdexcept>
namespace zappy {
    class Error : public std::runtime_error {
      public:
        Error(std::string message);
        ~Error() = default;
    };
}