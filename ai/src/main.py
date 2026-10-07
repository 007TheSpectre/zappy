import sys
from helpers.print_help import print_help
import helpers.interface as interface
import agents.winner
import agents.winner2
import agents.simple_lawn_mower
import agents.mover
import agents.chatterbox


for arg in sys.argv:
    if arg == "--help":
        print_help()
        sys.exit(0)

port_number = None
team_name = None
machine_name = "localhost"
agent_name = "default"

for i in range(len(sys.argv)):
    if sys.argv[i] == "-p" and i + 1 < len(sys.argv):
        port_number = int(sys.argv[i + 1])
        i += 1
    elif sys.argv[i] == "-n" and i + 1 < len(sys.argv):
        team_name = sys.argv[i + 1]
        i += 1
    elif sys.argv[i] == "-h" and i + 1 < len(sys.argv):
        machine_name = sys.argv[i + 1]
        i += 1
    elif sys.argv[i] == "-a" and i + 1 < len(sys.argv):
        agent_name = sys.argv[i + 1]
        i += 1

if port_number is None:
    print("Error: Port number is required.")
    sys.exit(84)
if team_name is None:
    print("Error: Team name is required.")
    sys.exit(84)

# conecting to the server

print(f"Connecting to {machine_name} on port {port_number} as team '{team_name}'...")

ai_interface = interface.AI_server_interface(machine_name, port_number, team_name)

if not ai_interface.connect():
    print("Failed to connect to the server.")
    sys.exit(84)
else:
    print("connected to the server")
    print(f"number of unused slots: {ai_interface.last_known_number_of_unused_slots}")
    print(f"world dimensions: {ai_interface.world_x_size} x {ai_interface.world_y_size}")

# creating the agent

if agent_name == "default":
    agent = agents.winner.WinnerAgent(ai_interface)
elif agent_name == "winner":
    agent = agents.winner.WinnerAgent(ai_interface)
elif agent_name == "winner2":
    agent = agents.winner2.WinnerAgent2(ai_interface)
elif agent_name == "simple_lawn_mower":
    agent = agents.simple_lawn_mower.SimpleLawnMowerAgent(ai_interface)
elif agent_name == "mover":
    agent = agents.mover.MoverAgent(ai_interface)
elif agent_name == "chatterbox":
    agent = agents.chatterbox.ChatterboxAgent(ai_interface)
else:
    print(f"Error: Unknown agent '{agent_name}'.")
    sys.exit(84)

# running the agent

agent.run()

print("Agent has stopped running.")
