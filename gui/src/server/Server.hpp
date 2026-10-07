/*
** EPITECH PROJECT, 2026
** zappy
** File description:
** Server
*/

#pragma once
#include "../network/Socket.hpp"
#include "../player/PlayerManager.hpp"
#include "../resources/ResourceTracker.hpp"
#include "../world/WorldState.hpp"
#include <mutex>
#include <vector>

namespace zappy {
    class Server {
      public:
        Server(Socket& socket) : _socket(socket) {}
        ~Server();
        void                           getInfo();
        void                           handleServerUpdate();
        void                           updateResources();
        void                           trashLogs(std::size_t maxSize);
        std::mutex&                    getMutex();
        const WorldState&              world() const;
        PlayerManager&                 players();
        const PlayerManager&           players() const;
        const ResourceTracker&         resources() const;
        const std::vector<std::string> logs() const;
        std::vector<Tile>              getTilesSafe() const;
        std::vector<Player>            getPlayersSafe() const;
        std::string                    resourceIDToString(int id) const;

      private:
        void                     dispatchLine(const std::vector<std::string>& words);
        void                     ensurePlayer(int number);
        void                     requestState(const std::vector<int>& alivePlayers);
        Socket&                  _socket;
        WorldState               _world;
        PlayerManager            _players;
        ResourceTracker          _resources;
        mutable std::mutex       _dataMutex;
        std::vector<std::string> _logs;
        std::string              _pending;
    };
}
