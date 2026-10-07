cmake_minimum_required(VERSION 3.16)
project(ZapfritesGui)

find_package(raylib REQUIRED)

add_executable(zappy_gui
    src/main.cpp
    src/core/Core.cpp
    src/errorhandler/Error.cpp
    src/network/Socket.cpp
    src/player/Player.cpp
    src/player/PlayerManager.cpp
    src/resources/ResourceTracker.cpp
    src/server/Server.cpp
    src/server/ServerParser.cpp
    src/world/Tile.cpp
    src/world/WorldState.cpp
    src/Renderer/Colors.cpp
    src/Renderer/Renderer.cpp
)

target_include_directories(zappy_gui PRIVATE src)
target_link_libraries(zappy_gui PRIVATE raylib)