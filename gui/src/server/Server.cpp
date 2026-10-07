/*
** EPITECH PROJECT, 2026
** zappy
** File description:
** Server
*/

#include "Server.hpp"
#include "ServerParser.hpp"
#include <cstdlib>
#include <mutex>
#include <string>
#include <vector>

namespace zappy {

    Server::~Server() {}
    // The server multiplexes asynchronous push events (ppo, pnw, pdi, pin...)
    // and request/reply payloads (bct from mct, plv, pin) onto one stream. A
    // single reader splits the stream into whole lines and dispatches each by
    // its prefix, so no message type can ever be swallowed by a reader that
    // only understands a different one.
    void Server::handleServerUpdate() {
        _pending += _socket.readFromSocket();

        std::vector<std::string> lines;
        std::size_t              start = 0;
        std::size_t              nl;
        while ((nl = _pending.find('\n', start)) != std::string::npos) {
            lines.push_back(_pending.substr(start, nl - start));
            start = nl + 1;
        }
        // Keep the trailing partial line (a message split across two reads) so
        // it is completed and parsed on the next pass instead of being lost.
        _pending.erase(0, start);

        std::vector<int> alivePlayers;
        {
            std::lock_guard<std::mutex> lock(_dataMutex);
            for (const std::string& line : lines) {
                const std::vector<std::string> words = ServerParser::splitWords(line);
                if (!words.empty())
                    dispatchLine(words);
            }
            _players.getAllAlive();
            const std::vector<Player>& players = _players.getPlayer();
            for (int i = 0; i < static_cast<int>(players.size()); ++i)
                if (players[i].getIsAlive())
                    alivePlayers.push_back(i);
        }
        requestState(alivePlayers);
    }

    // Routes one already-tokenised line to the right state update. Must be
    // called with _dataMutex held.
    void Server::dispatchLine(const std::vector<std::string>& words) {
        const std::string&   cmd     = words[0];
        std::vector<Player>& players = _players.getPlayer();

        if (cmd == "pbc" && words.size() >= 2) {
            const int         number = std::atoi(words[1].substr(1).c_str());
            const std::string msg    = words.size() >= 3 ? words[2] : "";
            ensurePlayer(number);
            _logs.push_back("Player " + std::to_string(number + 1) + " sent the message " + msg);
            _players.addBroadcaster(number);
        }
        if (cmd == "msz" && words.size() == 3) {
            const int x = std::atoi(words[1].c_str());
            const int y = std::atoi(words[2].c_str());
            _world.setMapSize(x, y);
            std::vector<Tile> newTiles;
            newTiles.reserve(x * y);
            for (int currY = 0; currY < y; ++currY) {
                for (int currX = 0; currX < x; ++currX) {
                    Tile t;
                    t.setPosition(currX, currY);
                    t.setRessources({0, 0, 0, 0, 0, 0, 0});
                    newTiles.push_back(t);
                }
            }
            _world.setTiles(newTiles);
            return;
        }
        if (cmd == "sgt" && words.size() == 2) {
            _world.setTimeReference(std::atoi(words[1].c_str()));
            return;
        }
        if ((cmd == "ppo" && words.size() == 5) || (cmd == "pnw" && words.size() == 7)) {
            const int number = std::atoi(words[1].substr(1).c_str());
            if (number < 0)
                return;
            ensurePlayer(number);
            ServerParser::parsePlayerPositionLine(words, players[number]);
            players[number].setIsAlive(true);
            if (cmd == "pnw") {
                std::string team = words[6];
                players[number].setTeam(team);
                players[number].setLevel(static_cast<u_int8_t>(std::atoi(words[5].c_str())));
                _logs.push_back("New player " + std::to_string(number + 1) + " connected");
            } else {
                _logs.push_back("Player " + std::to_string(number + 1) + " moved to position (" + words[2] + ", " + words[3] + ") facing " +
                                orientationToString(players[number].getPos().second));
            }
            return;
        }
        if (cmd == "pdi" && words.size() == 2) {
            const int number = std::atoi(words[1].substr(1).c_str());
            if (number >= 0 && static_cast<std::size_t>(number) < players.size())
                players[number].setIsAlive(false);
            _logs.push_back("Player " + std::to_string(number + 1) + " died");
            return;
        }
        if (cmd == "pgt" && words.size() == 3) {
            const int number   = std::atoi(words[1].substr(1).c_str());
            const int resource = std::atoi(words[2].c_str());
            _logs.push_back("Player " + std::to_string(number + 1) + " picked up " + resourceIDToString(resource));
            return;
        }
        if (cmd == "pex" && words.size() == 2) {
            const int number = std::atoi(words[1].substr(1).c_str());
            _logs.push_back("Player " + std::to_string(number + 1) + " tried to push others");
        }
        if (cmd == "pfk" && words.size() == 2) {
            const int number = std::atoi(words[1].substr(1).c_str());
            _logs.push_back("Player " + std::to_string(number + 1) + " laid an egg");
        }
        if (cmd == "edi" && words.size() == 2) {
            const int number = std::atoi(words[1].substr(1).c_str());
            _logs.push_back("Egg " + std::to_string(number + 1) + " was destroyed");
        }
        if (cmd == "pdr" && words.size() == 3) {
            const int number   = std::atoi(words[1].substr(1).c_str());
            const int resource = std::atoi(words[2].c_str());
            _logs.push_back("Player " + std::to_string(number + 1) + " dropped " + resourceIDToString(resource));
            return;
        }
        if (cmd == "pin" && words.size() >= 11) {
            const int number = std::atoi(words[1].substr(1).c_str());
            if (number < 0 || static_cast<std::size_t>(number) >= players.size())
                return;
            std::vector<int> res;
            for (int k = 4; k <= 10; ++k)
                res.push_back(std::atoi(words[k].c_str()));
            players[number].setInventory(res);
            return;
        }
        if (cmd == "plv" && words.size() == 3) {
            const int number = std::atoi(words[1].substr(1).c_str());
            if (number < 0 || static_cast<std::size_t>(number) >= players.size())
                return;
            const int lvl = std::atoi(words[2].c_str());
            if (lvl > players[number].getLevel()) {
                _players.addIncreaser(number);
                _logs.push_back("Player " + std::to_string(number + 1) + " increased Lvl to " + words[2]);
            }
            players[number].setLevel(static_cast<u_int8_t>(lvl));
            return;
        }
        if (cmd == "bct" && words.size() == 10) {
            const int        x = std::atoi(words[1].c_str());
            const int        y = std::atoi(words[2].c_str());
            std::vector<int> res;
            for (int k = 3; k <= 9; ++k)
                res.push_back(std::atoi(words[k].c_str()));
            _world.updateTile(x, y, res);
            return;
        }
    }

    std::string Server::resourceIDToString(int id) const {
        switch (id) {
            case 0: return "Food";
            case 1: return "Linemate";
            case 2: return "deraumere";
            case 3: return "Sibur";
            case 4: return "Mendiane";
            case 5: return "Phiras";
            default: return "Unknown";
        }
    }

    // Grows the player vector to include `number`, marking any newly created
    // gap entries as not-alive so phantom players are never rendered. Must be
    // called with _dataMutex held.
    void Server::ensurePlayer(int number) {
        std::vector<Player>& players = _players.getPlayer();
        if (static_cast<std::size_t>(number) < players.size())
            return;
        std::size_t oldSize = players.size();
        players.resize(static_cast<std::size_t>(number) + 1);
        for (std::size_t k = oldSize; k < players.size(); ++k)
            players[k].setIsAlive(false);
    }

    // Requests the state the server does NOT push spontaneously: the full map
    // (resources respawn silently), and each living player's level and
    // inventory (level-ups and food decay are not pushed). Replies arrive on
    // the same stream and are demultiplexed by handleServerUpdate next pass.
    void Server::requestState(const std::vector<int>& alivePlayers) {
        _socket.writeToSocket("mct\n");
        for (int i : alivePlayers) {
            _socket.writeToSocket("pin #" + std::to_string(i) + "\n");
            _socket.writeToSocket("plv #" + std::to_string(i) + "\n");
        }
    }

    void Server::updateResources() {
        std::vector<Tile> tiles;
        {
            std::lock_guard<std::mutex> lock(_dataMutex);
            tiles = _world.getTiles();
        }
        _resources.updateResources(tiles);
    }

    void Server::getInfo() {
        std::string mess = "GRAPHIC\n";
        _socket.writeToSocket(mess);
    }

    void Server::trashLogs(std::size_t maxSize) {
        std::lock_guard<std::mutex> lock(_dataMutex);
        if (_logs.size() > maxSize)
            _logs.erase(_logs.begin(), _logs.end() - static_cast<long>(maxSize));
    }

    std::mutex& Server::getMutex() {
        return _dataMutex;
    }

    const WorldState& Server::world() const {
        return _world;
    }

    PlayerManager& Server::players() {
        return _players;
    }

    const PlayerManager& Server::players() const {
        return _players;
    }

    const ResourceTracker& Server::resources() const {
        return _resources;
    }

    const std::vector<std::string> Server::logs() const {
        return _logs;
    }

    std::vector<Tile> Server::getTilesSafe() const {
        std::lock_guard<std::mutex> lock(_dataMutex);
        return _world.getTiles();
    }

    std::vector<Player> Server::getPlayersSafe() const {
        std::lock_guard<std::mutex> lock(_dataMutex);
        return _players.getPlayer();
    }
}
