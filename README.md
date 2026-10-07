# Zappy

A networked multiplayer game where autonomous AI clients compete on a shared tile
map. Players gather resources, survive by eating, and cooperate to reach level 8
through seven successive incantations — each requiring an exact combination of
stones and a minimum number of players on the same tile. The first team to bring
six players to level 8 wins.

Three separate programs communicate over a custom line-based TCP protocol: the
**server** owns the authoritative world state, the **GUI** renders a live view of
it, and the **AI** clients play the game on their own.

> **Epitech · `G-YEP-400` (« zappy »)** — team project built with
> [JordanMar1](https://github.com/JordanMar1),
> [SimonDutal](https://github.com/SimonDutal),
> [TheSpectre07](https://github.com/TheSpectre07),
> [guillaumematton](https://github.com/guillaumematton) and
> [Bat-J](https://github.com/Bat-J).
>
> **My role:** the network protocol and the server / AI side.
>
> This repository is my own copy of the assignment, published as a portfolio
> piece. The original repository is private.

---

## Components

| Directory | Language | Role |
|---|---|---|
| `server/` | Rust | Authoritative game state — map, resources, players, eggs, incantations, time management |
| `gui/` | C++ (CMake + raylib) | Graphical client rendering the world live |
| `ai/` | Python | Autonomous agents playing the game |

## The world

The map is a torus of `width × height` tiles. Each tile holds an amount of **food**
and the six minerals used by incantations: `linemate`, `deraumere`, `sibur`,
`mendiane`, `phiras` and `thystame`.

Every player carries an inventory, a direction and a level. Players broadcast
messages to each other, eject neighbours from a tile, and lay eggs — each egg
hatches into a new player slot for the team that laid it.

### Elevation requirements

| Level → | Players | linemate | deraumere | sibur | mendiane | phiras | thystame |
|---|---|---|---|---|---|---|---|
| 1 → 2 | 1 | 1 | – | – | – | – | – |
| 2 → 3 | 2 | 1 | 1 | 1 | – | – | – |
| 3 → 4 | 2 | 2 | – | 1 | – | 2 | – |
| 4 → 5 | 4 | 1 | 1 | 2 | – | 1 | – |
| 5 → 6 | 4 | 1 | 2 | 1 | 3 | – | – |
| 6 → 7 | 6 | 1 | 2 | 3 | – | 1 | – |
| 7 → 8 | 6 | 2 | 2 | 2 | 2 | 2 | 1 |

## Build

Requires `cargo`, `cmake`, `python3` and **raylib** (for the GUI).

```bash
make            # builds all three: server, GUI and AI
make server     # Rust server only  -> ./zappy_server
make gui        # C++ GUI only       -> ./zappy_gui
make ai         # Python AI only     -> ./zappy_ai (sets up ai/.venv)
make fclean     # remove the binaries
```

Each binary is copied to the repository root. A Nix flake (`flake.nix`,
`shell.nix`) is also provided for a reproducible development environment.

## Usage

```bash
./zappy_server -p <port> -x <width> -y <height> \
               -n <team1> <team2> ... -c <clientsPerTeam> -f <freq>

./zappy_gui  -p <port> -h <hostname>

./zappy_ai   -p <port> -h <hostname> -n <teamName>
```

| Flag | Meaning |
|---|---|
| `-p` | Port the server listens on |
| `-x` / `-y` | Map width and height |
| `-n` | Team names |
| `-c` | Number of slots available per team |
| `-f` | Server frequency (time units per second) |

### Example

```bash
# terminal 1
./zappy_server -p 4242 -x 20 -y 20 -n red blue -c 4 -f 100

# terminal 2 — as many agents as you like
./zappy_ai -p 4242 -h 127.0.0.1 -n red

# terminal 3
./zappy_gui -p 4242 -h 127.0.0.1
```

## AI agents

Several strategies ship with the project, selectable at launch with the `-a` flag:

| Agent | Behaviour |
|---|---|
| `winner` (default) | Tuned to win in most situations |
| `lawn_mower` | Sweeps the map in a lawn-mower pattern to collect everything |
| `pusher` | A `lawn_mower` that shoves encountered players out of the way |
| `chatterbox` | A `lawn_mower` that spams broadcast messages to disrupt other teams |
| `winner2` | Variant of `winner` with a slightly different strategy |

All agents are built on a shared `interface` class in `ai/src/helpers/` that
encapsulates the network protocol and exposes a Python API for the game. Adding a
new agent means dropping a class into `ai/src/agents/` — see the existing ones for
the shape.

## GUI controls

Each resource type is drawn as a coloured dot on the edge of its tile: red for
food, green `linemate`, blue `deraumere`, yellow `sibur`, purple `mendiane`,
orange `phiras`, pink `thystame`.

| Key | Action |
|---|---|
| `1` / `2` / `3` / `4` | Cycle tile background, tile border, background and text colours |
| `+` / `-` | Select next / previous player |
| Left click | Select the tile under the cursor |
| `F11` | Toggle fullscreen |
