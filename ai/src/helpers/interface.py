import helpers.networking as networking
from enum import Enum
from helpers.can_cast import can_cast
from random import choice, randint
import helpers.broadcast_strings as broadcast_strings


class object_type(Enum):
    player = "player"
    food = "food"
    egg = "egg"
    linemate = "linemate"
    deraumere = "deraumere"
    sibur = "sibur"
    mendiane = "mendiane"
    phiras = "phiras"
    thystame = "thystame"


def object_type_from_string(value: str) -> object_type:
    try:
        return object_type(value)
    except ValueError:
        raise ValueError(f"Invalid object type: {value}")


def str_to_object_type(value: str) -> object_type:
    return object_type(value)


class action_type(Enum):
    forward = "Forward"
    turn_left = "Left"
    turn_right = "Right"
    look = "Look"
    inventory = "Inventory"
    broadcast = "Broadcast"
    connect_nbr = "Connect_nbr"
    fork = "Fork"
    eject = "Eject"
    was_ejected = "Was_ejected"
    take = "Take"
    set_object_down = "Set"
    start_incantation = "Incantation"


class cardinal_direction(Enum):
    none = 0
    north = 1
    west = 2
    south = 3
    east = 4

    def right(self):
        if self == cardinal_direction.none:
            return cardinal_direction.none
        return cardinal_direction(((self.value + 2) % 4) + 1)

    def left(self):
        if self == cardinal_direction.none:
            return cardinal_direction.none
        return cardinal_direction((self.value % 4) + 1)


class direction(Enum):
    none = 0
    north = 1
    northwest = 2
    west = 3
    southwest = 4
    south = 5
    southeast = 6
    east = 7
    northeast = 8

    def __wrap_around(self, value: int) -> int:
        while value > 8:
            value -= 7
        while value < 1:
            value += 7
        return value

    def add_cardinal_direction(self, cardinal_direction: cardinal_direction):
        if cardinal_direction == cardinal_direction.none or self == direction.none:
            return direction.none
        rotation = cardinal_direction.value * 2 - 1
        return direction(self.__wrap_around(self.value + rotation - 1))

    def sub_cardinal_direction(self, cardinal_direction: cardinal_direction):
        if cardinal_direction == cardinal_direction.none or self == direction.none:
            return direction.none
        rotation = cardinal_direction.value * 2 - 1
        return direction(self.__wrap_around(self.value - rotation - 1))


def parse_direction(direction_value: int) -> direction:
    if direction_value == 0:
        return direction.none
    return direction(direction_value)


RESSOURSE_REQUIREMENTS_FOR_LEVEL_UP = {
    1: {
        object_type.linemate: 1,
        object_type.deraumere: 0,
        object_type.sibur: 0,
        object_type.mendiane: 0,
        object_type.phiras: 0,
        object_type.thystame: 0,
    },
    2: {
        object_type.linemate: 1,
        object_type.deraumere: 1,
        object_type.sibur: 1,
        object_type.mendiane: 0,
        object_type.phiras: 0,
        object_type.thystame: 0,
    },
    3: {
        object_type.linemate: 2,
        object_type.deraumere: 0,
        object_type.sibur: 1,
        object_type.mendiane: 0,
        object_type.phiras: 2,
        object_type.thystame: 0,
    },
    4: {
        object_type.linemate: 1,
        object_type.deraumere: 1,
        object_type.sibur: 2,
        object_type.mendiane: 0,
        object_type.phiras: 1,
        object_type.thystame: 0,
    },
    5: {
        object_type.linemate: 1,
        object_type.deraumere: 2,
        object_type.sibur: 1,
        object_type.mendiane: 3,
        object_type.phiras: 0,
        object_type.thystame: 0,
    },
    6: {
        object_type.linemate: 1,
        object_type.deraumere: 2,
        object_type.sibur: 3,
        object_type.mendiane: 0,
        object_type.phiras: 1,
        object_type.thystame: 0,
    },
    7: {
        object_type.linemate: 2,
        object_type.deraumere: 2,
        object_type.sibur: 2,
        object_type.mendiane: 2,
        object_type.phiras: 2,
        object_type.thystame: 1,
    },
}


PLAYER_COUNT_REQUIREMENTS_FOR_LEVEL_UP = {
    1: 1,
    2: 2,
    3: 2,
    4: 4,
    5: 4,
    6: 6,
    7: 6,
}


class AI_server_interface:
    def __init__(self, host: str, port: int, team_name: str):
        self.connection = networking.AI_server_connection(host, port)
        self.team_name = team_name
        self.port = port
        self.host = host
        self.world_x_size = None
        self.world_y_size = None
        self.alive = None
        self.last_known_number_of_unused_slots = None
        self.last_known_inventory = None  # todo
        self.level = 1
        self.pending_response_actions = []
        self.completed_actions = []
        self.message_history = []
        self.relative_direction = cardinal_direction.north
        self.relative_x_position = 0
        self.relative_y_position = 0

    def connect(self) -> bool:
        """Connects to the server and performs the initial handshake."""
        if not self.connection.connect():
            return False

        # server welcome message
        if not self.connection.blocking_receive():
            return False
        while self.connection.get_message() != "WELCOME\n":
            if not self.connection.blocking_receive():
                return False

        # send team name
        self.connection.send_message(self.team_name + "\n")

        # receive slot number
        if not self.connection.blocking_receive():
            return False
        received_message = self.connection.get_message()
        if received_message is None or received_message.strip() == "ko":
            return False
        self.last_known_number_of_unused_slots = int(received_message.strip())
        if self.last_known_number_of_unused_slots < 0:
            return False  # no slots available

        # receive world dimensions
        if not self.connection.blocking_receive():
            return False
        received_message = self.connection.get_message()
        if received_message is None or received_message.strip() == "ko":
            return False
        try:
            self.world_x_size, self.world_y_size = map(int, received_message.strip().split())
        except Exception:
            return False

        self.alive = True
        return True

    def is_connected(self) -> bool:
        """Checks if the connection to the server is still alive."""
        return self.connection.is_connected()

    def is_alive(self) -> bool:
        """Checks if the Trantorian is still alive"""
        if self.alive is None:
            return False
        else:
            return self.alive

    def __parse_look_response(self, response: str):
        response = response.strip()[1:-1].strip()
        tiles = response.split(",")
        parsed_response = []
        for tile in tiles:
            tile = tile.strip()
            if tile == "":
                parsed_response.append(
                    {
                        object_type.player: 0,
                        object_type.food: 0,
                        object_type.egg: 0,
                        object_type.linemate: 0,
                        object_type.deraumere: 0,
                        object_type.sibur: 0,
                        object_type.mendiane: 0,
                        object_type.phiras: 0,
                        object_type.thystame: 0,
                    }
                )
                continue
            objects = tile.split()
            object_counts = {}
            for obj in objects:
                if object_counts.get(object_type_from_string(obj)) is not None:
                    object_counts[object_type_from_string(obj)] += 1
                else:
                    object_counts[object_type_from_string(obj)] = 1
            parsed_response.append(object_counts)
        return parsed_response

    def __parse_inventory_response(self, response: str):
        response = response.strip()[1:-1].strip()
        items = response.split(",")
        inventory = {
            object_type.food: 0,
            object_type.linemate: 0,
            object_type.deraumere: 0,
            object_type.sibur: 0,
            object_type.mendiane: 0,
            object_type.phiras: 0,
            object_type.thystame: 0,
        }
        for item in items:
            item = item.strip()
            if item == "":
                continue
            try:
                obj, count = item.split()
                inventory[object_type_from_string(obj)] = int(count)
            except Exception:
                continue
        return inventory

    def __parse_message(self, message: str):
        if message is None:
            pass
        message = message.strip()

        if message == "dead":
            self.alive = False
            pass
        elif message == "ok":
            if self.pending_response_actions:
                if len(self.pending_response_actions) == 0:
                    return
                completed_action = self.pending_response_actions.pop(0)
                self.completed_actions.append({"action_type": completed_action, "success": True})
                if completed_action == action_type.turn_right:
                    self.relative_direction = self.relative_direction.right()
                elif completed_action == action_type.turn_left:
                    self.relative_direction = self.relative_direction.left()
                elif completed_action == action_type.forward:
                    if self.relative_direction == cardinal_direction.north:
                        self.relative_x_position += 1
                    elif self.relative_direction == cardinal_direction.east:
                        self.relative_y_position += 1
                    elif self.relative_direction == cardinal_direction.south:
                        self.relative_x_position -= 1
                    elif self.relative_direction == cardinal_direction.west:
                        self.relative_y_position -= 1
            pass
        elif message == "ko":
            if self.pending_response_actions:
                if len(self.pending_response_actions) == 0:
                    return
                completed_action = self.pending_response_actions.pop(0)
                self.completed_actions.append({"action_type": completed_action, "success": False})
            pass
        elif message.startswith("["):
            if message.startswith("[ player"):
                if len(self.pending_response_actions) == 0:
                    return
                completed_action = self.pending_response_actions.pop(0)
                self.completed_actions.append(
                    {
                        "action_type": completed_action,
                        "success": True,
                        "response": self.__parse_look_response(message),
                    }
                )
            else:
                if len(self.pending_response_actions) == 0:
                    return
                completed_action = self.pending_response_actions.pop(0)
                inventory = self.__parse_inventory_response(message)
                self.completed_actions.append(
                    {
                        "action_type": completed_action,
                        "success": True,
                        "response": inventory,
                    }
                )
                self.last_known_inventory = inventory
        elif can_cast(str, int, message) and int(message) >= 0:
            if len(self.pending_response_actions) == 0:
                return
            completed_action = self.pending_response_actions.pop(0)
            self.completed_actions.append(
                {
                    "action_type": completed_action,
                    "success": True,
                    "response": int(message),
                }
            )
            if completed_action == action_type.connect_nbr:
                self.last_known_number_of_unused_slots = int(message)
            pass
        elif message == "Elevation underway":
            pass
        elif message.startswith("Current level: "):
            # "Current level: N" can arrive unsolicited: another player's
            # incantation elevates everyone of that level on the tile, including
            # us while we're idle with no pending Incantation of our own. Always
            # apply the new level; only consume a pending action if this was our
            # own cast completing. The old early-return on an empty queue dropped
            # passive elevations, leaving the player stuck at its previous level.
            if self.pending_response_actions and self.pending_response_actions[0] == action_type.start_incantation:
                self.pending_response_actions.pop(0)
            self.level = int(message.split(":")[1].strip())
            self.completed_actions.append(
                {
                    "action_type": action_type.start_incantation,
                    "success": True,
                    "response": self.level,
                }
            )
            pass
        elif message.startswith("message "):
            self.message_history.append(
                {
                    "text": message[11:],
                    "direction": parse_direction(int(message[8:9])).add_cardinal_direction(self.relative_direction),
                    "raw_direction": parse_direction(int(message[8:9])),
                }
            )
            pass
        elif message.startswith("eject: "):
            direction = parse_direction(int(message[7:])).add_cardinal_direction(self.relative_direction)
            if direction == direction.north:
                self.relative_x_position -= 1
            elif direction == direction.northwest:
                self.relative_x_position -= 1
                self.relative_y_position -= 1
            elif direction == direction.west:
                self.relative_y_position -= 1
            elif direction == direction.southwest:
                self.relative_x_position += 1
                self.relative_y_position -= 1
            elif direction == direction.south:
                self.relative_x_position += 1
            elif direction == direction.southeast:
                self.relative_x_position += 1
                self.relative_y_position += 1
            elif direction == direction.east:
                self.relative_y_position += 1
            elif direction == direction.northeast:
                self.relative_x_position -= 1
                self.relative_y_position += 1
            self.completed_actions.append(
                {
                    "action_type": action_type.was_ejected,
                    "direction": direction,
                }
            )
            pass

    def orient_to_direction(self, direction: cardinal_direction):
        """Orients the Trantorian to the specified cardinal direction (relative to its starting orientation)."""
        if self.relative_direction == direction:
            return
        elif (self.relative_direction.value - direction.value) % 4 == 3:
            self.send_turn_left_action_and_block_until_response()
        elif (self.relative_direction.value - direction.value) % 4 == 1:
            self.send_turn_right_action_and_block_until_response()
        else:
            self.send_turn_left_action_and_block_until_response()
            self.send_turn_left_action_and_block_until_response()

    def goto(self, x_target: int, y_target: int):
        """Moves the Trantorian to the specified relative coordinates (relative to its starting position)."""
        if self.relative_x_position < x_target:
            self.orient_to_direction(cardinal_direction.north)
            for _ in range(x_target - self.relative_x_position):
                self.send_forward_action_and_block_until_response()
        elif self.relative_x_position > x_target:
            self.orient_to_direction(cardinal_direction.south)
            for _ in range(self.relative_x_position - x_target):
                self.send_forward_action_and_block_until_response()
        if self.relative_y_position < y_target:
            self.orient_to_direction(cardinal_direction.east)
            for _ in range(y_target - self.relative_y_position):
                self.send_forward_action_and_block_until_response()
        elif self.relative_y_position > y_target:
            self.orient_to_direction(cardinal_direction.west)
            for _ in range(self.relative_y_position - y_target):
                self.send_forward_action_and_block_until_response()

    def move_towards_absolute_direction(self, direction: direction):
        """Moves the Trantorian one tile in the specified direction (relative to its starting orientation)."""
        if direction == direction.north:
            self.orient_to_direction(cardinal_direction.north)
            self.send_forward_action_and_block_until_response()
        elif direction == direction.northwest:
            self.orient_to_direction(cardinal_direction.north)
            self.send_forward_action_and_block_until_response()
            self.orient_to_direction(cardinal_direction.west)
            self.send_forward_action_and_block_until_response()
        elif direction == direction.west:
            self.orient_to_direction(cardinal_direction.west)
            self.send_forward_action_and_block_until_response()
        elif direction == direction.southwest:
            self.orient_to_direction(cardinal_direction.south)
            self.send_forward_action_and_block_until_response()
            self.orient_to_direction(cardinal_direction.west)
            self.send_forward_action_and_block_until_response()
        elif direction == direction.south:
            self.orient_to_direction(cardinal_direction.south)
            self.send_forward_action_and_block_until_response()
        elif direction == direction.southeast:
            self.orient_to_direction(cardinal_direction.south)
            self.send_forward_action_and_block_until_response()
            self.orient_to_direction(cardinal_direction.east)
            self.send_forward_action_and_block_until_response()
        elif direction == direction.east:
            self.orient_to_direction(cardinal_direction.east)
            self.send_forward_action_and_block_until_response()
        elif direction == direction.northeast:
            self.orient_to_direction(cardinal_direction.north)
            self.send_forward_action_and_block_until_response()
            self.orient_to_direction(cardinal_direction.east)
            self.send_forward_action_and_block_until_response()

    def collect_tile_ressources(self):
        """Collects all ressources on the current tile. blocking"""
        self.send_look_action_and_block_until_response()
        if (
            self.completed_actions[-1]["action_type"] != action_type.look
            or not self.completed_actions[-1]["success"]
            or not self.completed_actions[-1]["response"]
        ):
            return
        for obj, count in self.completed_actions[-1]["response"][0].items():
            if obj != object_type.player and obj != object_type.egg and count > 0:
                for _ in range(count):
                    self.send_take_action(obj)
        self.block_until_all_responses_received()

    def collect_matching_tile_ressources(self, ressources: dict):
        """Collects the specified amount of ressources on the current tile. blocking"""
        self.send_look_action_and_block_until_response()
        last_action = self.completed_actions[-1]
        if (
            self.completed_actions[-1]["action_type"] != action_type.look
            or not last_action.get("success")
            or not last_action.get("response")
            or not isinstance(last_action.get("response"), list)
        ):
            return
        for obj, count in self.completed_actions[-1]["response"][0].items():
            if obj != object_type.player and obj != object_type.egg and count > 0:
                needed_count = ressources.get(obj, 0)
                if needed_count > 0:
                    for _ in range(min(count, needed_count)):
                        self.send_take_action(obj)
        self.block_until_all_responses_received()

    def are_ressources_sufficient_for_level_up(self, amount: dict, needed_amount: dict) -> bool:
        """Checks if the Trantorian has enough ressources in its inventory for the next level."""
        if amount is None or needed_amount is None:
            return False
        for resource, required_amount in needed_amount.items():
            if amount.get(resource, 0) < required_amount:
                return False
        return True

    def has_enough_ressources_for_next_level(self) -> bool:
        """Checks if the Trantorian has enough ressources in its inventory for the next level."""
        if self.last_known_inventory is None:
            return False
        requirements = RESSOURSE_REQUIREMENTS_FOR_LEVEL_UP.get(self.level, None)
        if requirements is None:
            return False
        return self.are_ressources_sufficient_for_level_up(self.last_known_inventory, requirements)

    def get_ressources_needed_for_level(self, level: int) -> dict:
        """Returns a dictionary of the ressources needed for the specified level."""
        return RESSOURSE_REQUIREMENTS_FOR_LEVEL_UP.get(level, {})

    def get_missing_ressources_for_next_level(self, inventory: dict) -> dict:
        """Returns a dictionary of the missing ressources needed for the next level."""
        if inventory is None:
            return {}
        requirements = RESSOURSE_REQUIREMENTS_FOR_LEVEL_UP.get(self.level, None)
        if requirements is None:
            return {}
        missing = {}
        for resource, required_amount in requirements.items():
            current_amount = inventory.get(resource, 0)
            if current_amount < required_amount:
                missing[resource] = required_amount - current_amount
        return missing

    def add_objects_dicts(self, dict1: dict, dict2: dict) -> dict:
        """Adds the counts of two dictionaries of object types and returns the result."""
        result = dict1.copy()
        for obj, count in dict2.items():
            result[obj] = result.get(obj, 0) + count
        return result

    def subtract_objects_dicts(self, dict1: dict, dict2: dict) -> dict:
        """Subtracts the counts of two dictionaries of object types and returns the result."""
        result = dict1.copy()
        for obj, count in dict2.items():
            result[obj] = result.get(obj, 0) - count
            if result[obj] < 0:
                result[obj] = 0
        return result

    def handle_server_messages_blocking(self):
        """Handles incoming messages from the server and updates the internal state accordingly."""
        self.connection.blocking_receive()
        while self.connection.is_message_available():
            message = self.connection.get_message()
            if message is not None:
                self.__parse_message(message)

    def handle_server_messages_nonblocking(self):
        """Handles incoming messages from the server and updates the internal state accordingly."""
        self.connection.nonblocking_receive()
        while self.connection.is_message_available():
            message = self.connection.get_message()
            if message is not None:
                self.__parse_message(message)

    def block_until_all_responses_received(self):
        """Blocks until all pending response actions have received a response from the server."""
        while self.pending_response_actions and self.connection.is_connected() and self.is_alive():
            self.handle_server_messages_blocking()

    def send_forward_action(self):
        """Sends a forward action to the server."""
        self.connection.send_message(action_type.forward.value + "\n")
        self.pending_response_actions.append(action_type.forward)

    def send_turn_right_action(self):
        """Sends a turn right action to the server."""
        self.connection.send_message(action_type.turn_right.value + "\n")
        self.pending_response_actions.append(action_type.turn_right)

    def send_turn_left_action(self):
        """Sends a turn left action to the server."""
        self.connection.send_message(action_type.turn_left.value + "\n")
        self.pending_response_actions.append(action_type.turn_left)

    def send_look_action(self):
        """Sends a look action to the server."""
        self.connection.send_message(action_type.look.value + "\n")
        self.pending_response_actions.append(action_type.look)

    def send_inventory_action(self):
        """Sends an inventory action to the server."""
        self.connection.send_message(action_type.inventory.value + "\n")
        self.pending_response_actions.append(action_type.inventory)

    def send_broadcast_action(self, message: str):
        """Sends a broadcast action to the server with the specified message."""
        self.connection.send_message(f"{action_type.broadcast.value} {message}\n")
        self.pending_response_actions.append(action_type.broadcast)

    def send_connect_nbr_action(self):
        """Sends a connect_nbr action to the server."""
        self.connection.send_message(action_type.connect_nbr.value + "\n")
        self.pending_response_actions.append(action_type.connect_nbr)

    def send_fork_action(self):
        """Sends a fork action to the server."""
        self.connection.send_message(action_type.fork.value + "\n")
        self.pending_response_actions.append(action_type.fork)

    def send_eject_action(self):
        """Sends an eject action to the server."""
        self.connection.send_message(action_type.eject.value + "\n")
        self.pending_response_actions.append(action_type.eject)

    def send_take_action(self, object_type: object_type):
        """Sends a take action to the server for the specified object."""
        self.connection.send_message(f"{action_type.take.value} {object_type.value}\n")
        self.pending_response_actions.append(action_type.take)

    def send_set_object_down_action(self, object_type: object_type):
        """Sends a set_object_down action to the server for the specified object."""
        self.connection.send_message(f"{action_type.set_object_down.value} {object_type.value}\n")
        self.pending_response_actions.append(action_type.set_object_down)

    def send_start_incantation_action(self):
        """Sends a start_incantation action to the server."""
        self.connection.send_message(f"{action_type.start_incantation.value}\n")
        self.pending_response_actions.append(action_type.start_incantation)

    def send_forward_action_and_block_until_response(self):
        """Sends a forward action to the server and blocks until a response is received."""
        self.send_forward_action()
        self.block_until_all_responses_received()
        return self.completed_actions[-1]

    def send_turn_right_action_and_block_until_response(self):
        """Sends a turn right action to the server and blocks until a response is received."""
        self.send_turn_right_action()
        self.block_until_all_responses_received()
        return self.completed_actions[-1]

    def send_turn_left_action_and_block_until_response(self):
        """Sends a turn left action to the server and blocks until a response is received."""
        self.send_turn_left_action()
        self.block_until_all_responses_received()
        return self.completed_actions[-1]

    def send_look_action_and_block_until_response(self):
        """Sends a look action to the server and blocks until a response is received."""
        self.send_look_action()
        self.block_until_all_responses_received()
        return self.completed_actions[-1]

    def send_inventory_action_and_block_until_response(self):
        """Sends an inventory action to the server and blocks until a response is received."""
        self.send_inventory_action()
        self.block_until_all_responses_received()
        return self.completed_actions[-1]

    def send_broadcast_action_and_block_until_response(self, message: str):
        """Sends a broadcast action to the server with the specified message and blocks until a response is received."""
        self.send_broadcast_action(message)
        self.block_until_all_responses_received()
        return self.completed_actions[-1]

    def send_connect_nbr_action_and_block_until_response(self):
        """Sends a connect_nbr action to the server and blocks until a response is received."""
        self.send_connect_nbr_action()
        self.block_until_all_responses_received()
        return self.completed_actions[-1]

    def send_fork_action_and_block_until_response(self):
        """Sends a fork action to the server and blocks until a response is received."""
        self.send_fork_action()
        self.block_until_all_responses_received()
        return self.completed_actions[-1]

    def send_eject_action_and_block_until_response(self):
        """Sends an eject action to the server and blocks until a response is received."""
        self.send_eject_action()
        self.block_until_all_responses_received()
        return self.completed_actions[-1]

    def send_take_action_and_block_until_response(self, object_type: object_type):
        """Sends a take action to the server for the specified object and blocks until a response is received."""
        self.send_take_action(object_type)
        self.block_until_all_responses_received()
        return self.completed_actions[-1]

    def send_set_object_down_action_and_block_until_response(self, object_type: object_type):
        """Sends a set_object_down action to the server for the specified object and blocks until a response is received."""
        self.send_set_object_down_action(object_type)
        self.block_until_all_responses_received()
        return self.completed_actions[-1]

    def send_start_incantation_action_and_block_until_response(self):
        """Sends a start_incantation action to the server and blocks until a response is received."""
        self.send_start_incantation_action()
        self.block_until_all_responses_received()
        return self.completed_actions[-1]

    def sanitize_broadcast_message(self, message: str) -> str:
        """sanitizes a broadcast"""
        return message.replace(" ", "_").replace("\n", "_")

    def generate_fake_broadcast_random_message(self) -> str:
        return choice(broadcast_strings.RANDOM_MESSAGES)

    def generate_fake_broadcast_response_message(self, message: str) -> str:
        random_action = randint(0, 9)
        if random_action == 0:
            # return the message as is
            return message
        elif random_action == 1 and any(char.isdigit() for char in message):
            # replace digits in the message with random digits
            for char in message:
                if char.isdigit():
                    random_digit = randint(0, 9)
                    message = message.replace(char, str(random_digit), 1)
            return message
        elif random_action == 2:
            # censor a random word in the message
            words = message.split("_")
            if words:
                random_word_index = randint(0, len(words) - 1)
                words[random_word_index] = "******"
                return "_".join(words)
        elif random_action == 3:
            # put the message in uppercase
            return message.upper()
        elif random_action == 4:
            # put the message in lowercase
            return message.lower()
        elif random_action == 5:
            # return a random number
            return str(randint(0, 99))
        elif random_action == 6:
            # cesar code the message with a random shift
            shift = randint(1, 25)
            encrypted_message = ""
            for char in message:
                if char.isalpha():
                    if char.isupper():
                        offset = 65
                    else:
                        offset = 97
                    encrypted_char = chr((ord(char) - offset + shift) % 26 + offset)
                    encrypted_message += encrypted_char
                else:
                    encrypted_message += char
            return encrypted_message
        elif random_action == 7:
            # reverse message
            return message[::-1]
        elif random_action == 8:
            # replace vowels with consonants and vice versa
            vowels = "aeiouyAEIOUY"
            consonants = "bcdfghjklmnpqrstvwxyzBCDFGHJKLMNPQRSTVWXYZ"
            for vowel in vowels:
                message = message.replace(vowel, consonants[randint(0, len(consonants) - 1)])
            return message
        elif random_action == 9:
            # replace consonants with vowels and vice versa
            vowels = "aeiouyAEIOUY"
            consonants = "bcdfghjklmnpqrstvwxyzBCDFGHJKLMNPQRSTVWXYZ"
            for consonant in consonants:
                message = message.replace(consonant, vowels[randint(0, len(vowels) - 1)])
            return message
        return message

    def generate_fake_broadcast_message(self, message: str = "") -> str:
        """Generates a broadcast message meant to confuse other agents. Message parameter is optional"""
        output = ""
        if message is not None and message.strip() != "":
            # 2/3 response, 1/3 random
            if randint(0, 2) != 0:
                output = self.generate_fake_broadcast_response_message(message)
            else:
                output = self.generate_fake_broadcast_random_message()
        else:
            output = self.generate_fake_broadcast_random_message()
        return self.sanitize_broadcast_message(output)
