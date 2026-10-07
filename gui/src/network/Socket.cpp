/*
** EPITECH PROJECT, 2026
** G-YEP-400-LIL-4-1-zappy-2
** File description:
** Socket
*/

#include "Socket.hpp"
#include "../errorhandler/Error.hpp"

#include <sys/socket.h>
#include <netinet/in.h>
#include <poll.h>
#include <unistd.h>
#include <arpa/inet.h>
#include <signal.h>
#include <cstring>
#include <cerrno>

namespace zappy {

    Socket::Socket() : _socketFD(-1) {}

    Socket::~Socket() {
        closeSocket();
    }

    void Socket::closeSocket() {
        if (_socketFD != -1) {
            shutdown(_socketFD, SHUT_RDWR);
            close(_socketFD);
            _socketFD = -1;
        }
    }

    void Socket::writeToSocket(const std::string& message) {
        write(_socketFD, message.c_str(), message.size());
    }

    std::string Socket::readFromSocket() {
        std::string message;
        char        chunk[1024];
        pollfd      pollFD{_socketFD, POLLIN, 0};

        while (true) {
            if (poll(&pollFD, 1, 0) <= 0)
                break;
            const ssize_t bytesRead = recv(_socketFD, chunk, sizeof(chunk) - 1, MSG_DONTWAIT);
            if (bytesRead < 0) {
                if (errno == EAGAIN || errno == EWOULDBLOCK || errno == EINTR)
                    continue;
                throw Error("Failed to read from socket");
            }
            if (bytesRead == 0)
                break;
            chunk[bytesRead] = '\0';
            message.append(chunk, static_cast<std::size_t>(bytesRead));
        }
        return message;
    }

    bool Socket::connectTo(const uint32_t ipAddress, uint16_t port) {
        closeSocket();
        _socketFD = socket(AF_INET, SOCK_STREAM, 0);
        if (_socketFD == -1) {
            throw Error("Failed to create socket");
        }
        struct sockaddr_in serverAddress;
        serverAddress.sin_family      = AF_INET;
        serverAddress.sin_addr.s_addr = ipAddress;
        serverAddress.sin_port        = htons(port);
        if (connect(_socketFD, reinterpret_cast<sockaddr*>(&serverAddress), sizeof(serverAddress)) == -1) {
            return false;
        }
        return true;
    }

    bool Socket::CheckIfOpen() const {
        if (_socketFD == -1) {
            return false;
        }
        struct sigaction emptySignalHandler;
        struct sigaction oldSignalHandler;
        emptySignalHandler.sa_handler = SIG_IGN;
        sigemptyset(&emptySignalHandler.sa_mask);
        emptySignalHandler.sa_flags = 0;
        sigaction(SIGPIPE, &emptySignalHandler, &oldSignalHandler);
        if (write(_socketFD, "", 0) == -1 && errno != EBADF) {
            sigaction(SIGPIPE, &oldSignalHandler, NULL);
            return false;
        } else {
            sigaction(SIGPIPE, &oldSignalHandler, NULL);
            return true;
        }
    }
}
