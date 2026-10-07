/*
** EPITECH PROJECT, 2026
** zappy
** File description:
** ServerParser
*/

#include "ServerParser.hpp"
#include <sstream>

namespace zappy {
    std::vector<std::string> ServerParser::splitWords(const std::string& input) {
        std::vector<std::string> words;
        std::stringstream        stream(input);
        std::string              word;

        while (stream >> word) {
            words.push_back(word);
        }
        return words;
    }

    Orientation ServerParser::parseOrientation(const std::string& token) {
        // The server encodes orientation as a letter (N/E/S/W). Digit forms
        // (1-4) are accepted too so the parser stays robust to either encoding.
        if (token.empty())
            return NORTH;
        switch (token[0]) {
            case '1': return NORTH;
            case '2': return EAST;
            case '3': return SOUTH;
            case '4': return WEST;
            default: return NORTH;
        }
    }

    bool ServerParser::parsePlayerPositionLine(const std::vector<std::string>& words, Player& player) {
        // Expected: "ppo #id X Y O" -> 5 tokens. Bail out on a truncated or
        // malformed line instead of indexing past the end of the vector (UB).
        if (words.size() < 5)
            return false;
        std::pair<std::pair<int, int>, Orientation> pos;
        pos.first.first  = std::atoi(words[2].c_str());
        pos.first.second = std::atoi(words[3].c_str());
        pos.second       = static_cast<Orientation>(std::atoi(words[4].c_str()));
        player.setPos(pos);
        return true;
    }
}