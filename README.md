> **Epitech project — `G-YEP-400` (`zappy`)**
>
> Built with [JordanMar1](https://github.com/JordanMar1), [SimonDutal](https://github.com/SimonDutal), [TheSpectre07](https://github.com/TheSpectre07), [guillaumematton](https://github.com/guillaumematton), [Bat-J](https://github.com/Bat-J).
> I worked on the network protocol and the server/AI side of the project.
>
> This is my own copy of the assignment repository, published here as a
> portfolio piece. The original repository is private.

---

# Zapfrites

# Usage

```
./zappy_server -p [port] -x [width] -y [height] -n [team_name1 team_name2 ...] -c [clientsNb] -f [freq]
```

```
./zappy_gui -p [port] -h [hostname]
```

```
./zappy_ai -p [port] -h [hostname] -n [team_name]
```

## Gui

each circle of the sides of a tile represent one or more ressources of a type.
ressources types:
- red: food
- green: linemate
- blue: deraumere
- yellow: sibur
- purple: mendiane
- orange: phiras
- pink: thystame

commands:
- ```1``` cycle through tile background colors
- ```2``` cycle through tile border colors
- ```3``` cycle through background colors
- ```4``` cycle through text colors
- ```+``` select next player
- ```-``` select previous player
- ```LMB``` select tile hovered by mouse
- ```F11``` toggle fullscreen

## AI

Different AI implementations have been created in order to test out features and try out different strategies.  
When starting an appy_ai intsance, you can specify the agent by using the `-a` flag followed by the agent name.  
Here are the available agents:
- winner (default): ai agent made to win the game in most situations  
- lawn_mower: ai agent that moves around the map in a lawn mower pattern to collect all ressources
- pusher: a lawn mower agent that pushes other encountered players out of the way
- chatterbox: a lawn mower agent that sends disruptive broadcast messages spontaneously or in response to other broadcasts
- winner2: a modified version of the winner agent that uses a slightly different strategy to win the game

AIs where implemented using a 'interface' class that encapsulates the basic network logic and provides an API for the AI agents to interact with the game server.  
You can add new AI agents by creating a new agent class similar to the existing ones in the `ai/src/agents` directory and implementing your own logic for the agent's behavior.  