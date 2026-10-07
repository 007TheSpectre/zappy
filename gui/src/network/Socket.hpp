/*
** EPITECH PROJECT, 2026
** G-NWP-400-LIL-4-1-myftp-5
** File description:
** Socket
*/

#pragma once

#include <string>
#include <cstdint>

namespace zappy {

    class Socket {
      public:
        Socket();
        ~Socket();
        void        closeSocket();
        void        writeToSocket(const std::string& message);
        std::string readFromSocket();
        bool        connectTo(const uint32_t ipAddress, uint16_t port);
        bool        CheckIfOpen() const;
        int         getSocket() const {
            return _socketFD;
        }
        int getPort() const {
            return _port;
        }

      private:
        int      _socketFD;
        uint16_t _port;
    };
}