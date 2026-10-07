/*
** EPITECH PROJECT, 2026
** gui main
** File description:
** GUI
*/

#include <iostream>
#include <string>
#include <arpa/inet.h>
#include <netdb.h>
#include <cstring>
#include <unistd.h>
#include <limits.h>
#include "network/Socket.hpp"
#include "core/Core.hpp"

static void chdirToAssets() {
    char    exe[PATH_MAX];
    ssize_t len = readlink("/proc/self/exe", exe, sizeof(exe) - 1);
    if (len == -1)
        return;
    exe[len]    = '\0';
    char* slash = strrchr(exe, '/');
    if (slash)
        *slash = '\0';
    // Deployed: binary at repo root, assets at <exe_dir>/gui/assets/
    char candidate[PATH_MAX];
    snprintf(candidate, sizeof(candidate), "%s/gui", exe);
    if (access(candidate, F_OK) == 0) {
        chdir(candidate);
        return;
    }
    // Built in place: binary at gui/build/, assets at <exe_dir>/../assets/
    snprintf(candidate, sizeof(candidate), "%s/..", exe);
    chdir(candidate);
}

int main(int argc, char** argv) {
    chdirToAssets();
    if (argc == 2 && std::string(argv[1]) == "--help") {
        std::cout << "Usage: ./zappy_gui -p port -h machine\n";
        return 0;
    } else if (argc != 5 || std::string(argv[1]) != "-p" || std::string(argv[3]) != "-h") {
        std::cerr << "Usage: ./zappy_gui -p port -h machine\n";
        return 84;
    }
    struct addrinfo  hints{};
    struct addrinfo* res = nullptr;
    hints.ai_family      = AF_INET;
    hints.ai_socktype    = SOCK_STREAM;
    if (getaddrinfo(argv[4], nullptr, &hints, &res) != 0 || res == nullptr) {
        std::cerr << "Failed to resolve host: " << argv[4] << "\n";
        return 84;
    }
    uint32_t ipAddress = reinterpret_cast<struct sockaddr_in*>(res->ai_addr)->sin_addr.s_addr;
    freeaddrinfo(res);
    zappy::Socket socket;
    if (!socket.connectTo(ipAddress, std::stoi(argv[2]))) {
        std::cerr << "Failed to connect to server\n";
        return 84;
    }
    zappy::Core core{};
    return core.Run(socket);
}
