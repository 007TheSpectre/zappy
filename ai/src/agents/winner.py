import helpers.interface as interface
from enum import Enum
from random import random
from helpers.can_cast import can_cast
from subprocess import Popen
from os import getcwd


cwd = getcwd()


class WinnerAgent:
    class State(Enum):
        none = 0
        collecting_ressources = 1
        egg_laying = 2
        active_elevating = 3
        passive_elevating = 4

    def __init__(self, ai_interface: interface.AI_server_interface):
        self.ai_interface = ai_interface
        self.state = WinnerAgent.State.none
        self.elevation_ressources_dropped = False
        self.elevation_broadcast_direction = None
        self.need_for_more_players = 0

        self.low_food_elevation_abort_threshold = 10
        self.minimum_food_for_active_elevation = 25
        self.minimum_food_for_passive_elevation = 20
        self.minimum_food_for_egg_laying = 15

    def run(self):
        while True:
            if not self.ai_interface.is_connected():
                print("Connection to the server lost.")
                break
            self.ai_interface.handle_server_messages_nonblocking()
            if not self.ai_interface.is_alive():
                print("Trantorian is no longer alive.")
                break

            if self.need_for_more_players >= self.ai_interface.level * 2:
                print("Spawning a new player to help with elevation...")
                Popen(
                    [
                        "/bin/bash",
                        f"{cwd}/zappy_ai",
                        "-h",
                        self.ai_interface.host,
                        "-p",
                        str(self.ai_interface.port),
                        "-n",
                        self.ai_interface.team_name,
                        "-a",
                        "winner",
                    ],
                    start_new_session=False,
                )
                self.need_for_more_players = 0

            for message in self.ai_interface.message_history:
                if (
                    message["text"].startswith("wawa_")
                    and can_cast(str, int, message["text"][5:])
                    and int(message["text"][5:]) == self.ai_interface.level
                ):
                    if (
                        self.elevation_broadcast_direction != interface.direction.none
                    ):  # message from the same tile already received, avoiding moving out
                        self.elevation_broadcast_direction = message["direction"]
                self.ai_interface.message_history.remove(message)

            if self.state == WinnerAgent.State.none:
                self.ai_interface.send_inventory_action()
                self.ai_interface.send_connect_nbr_action_and_block_until_response()
                if (
                    self.ai_interface.last_known_number_of_unused_slots is not None
                    and self.ai_interface.last_known_number_of_unused_slots == 0
                    and self.ai_interface.last_known_inventory is not None
                    and self.ai_interface.last_known_inventory[interface.object_type.food]
                    >= self.minimum_food_for_egg_laying
                ):
                    self.state = WinnerAgent.State.egg_laying
                elif (
                    self.elevation_broadcast_direction is None  # no one is active elevating
                    and self.ai_interface.has_enough_ressources_for_next_level()
                    and self.ai_interface.last_known_inventory is not None
                    and self.ai_interface.last_known_inventory[interface.object_type.food]
                    >= self.minimum_food_for_active_elevation
                ):
                    self.elevation_ressources_dropped = False
                    self.state = WinnerAgent.State.active_elevating
                elif (
                    self.elevation_broadcast_direction is not None  # someone is already active elevating
                    and self.ai_interface.last_known_inventory is not None
                    and self.ai_interface.last_known_inventory[interface.object_type.food]
                    >= self.minimum_food_for_passive_elevation
                ):
                    self.state = WinnerAgent.State.passive_elevating
                else:
                    self.state = WinnerAgent.State.collecting_ressources

            if self.state == WinnerAgent.State.collecting_ressources:
                if (
                    random()
                    < 1
                    / min(
                        self.ai_interface.world_x_size,  # pyright: ignore[reportArgumentType]
                        self.ai_interface.world_y_size,  # pyright: ignore[reportArgumentType]
                    )
                    * 2
                ):
                    if random() < 0.5:
                        self.ai_interface.send_turn_left_action()
                    else:
                        self.ai_interface.send_turn_right_action()
                self.ai_interface.send_forward_action_and_block_until_response()
                self.ai_interface.send_inventory_action_and_block_until_response()
                # ressources needed for the next two incantations
                needed_ressources = self.ai_interface.subtract_objects_dicts(
                    self.ai_interface.add_objects_dicts(
                        self.ai_interface.get_ressources_needed_for_level(self.ai_interface.level + 1),
                        self.ai_interface.get_ressources_needed_for_level(self.ai_interface.level),
                    ),
                    self.ai_interface.last_known_inventory,  # pyright: ignore[reportArgumentType]
                )
                needed_ressources[interface.object_type.food] = 100  # collect all
                self.ai_interface.collect_matching_tile_ressources(needed_ressources)
                self.state = WinnerAgent.State.none

            elif self.state == WinnerAgent.State.egg_laying:
                self.ai_interface.send_fork_action_and_block_until_response()
                self.state = WinnerAgent.State.none

            elif self.state == WinnerAgent.State.active_elevating:
                self.ai_interface.send_inventory_action_and_block_until_response()
                if (
                    self.ai_interface.last_known_inventory
                    and self.ai_interface.last_known_inventory[interface.object_type.food]
                    < self.low_food_elevation_abort_threshold
                ):  # critically low on food abort, also means the elavation did not succeed
                    self.state = WinnerAgent.State.none
                    self.need_for_more_players += 1

                if not self.elevation_ressources_dropped:
                    for (
                        resource,
                        required_amount,
                    ) in interface.RESSOURSE_REQUIREMENTS_FOR_LEVEL_UP.get(self.ai_interface.level, {}).items():
                        current_amount = (
                            self.ai_interface.completed_actions[-1]["response"][0].get(resource, 0)
                            if self.ai_interface.completed_actions
                            and self.ai_interface.completed_actions[-1]["action_type"] == interface.action_type.look
                            and self.ai_interface.completed_actions[-1]["response"]
                            and self.ai_interface.completed_actions[-1]["response"][0]
                            else 0
                        )
                        if required_amount > current_amount:
                            for _ in range(required_amount - current_amount):
                                self.ai_interface.send_set_object_down_action_and_block_until_response(resource)
                    self.elevation_ressources_dropped = True

                self.ai_interface.send_start_incantation_action_and_block_until_response()
                if (
                    self.ai_interface.completed_actions
                    and self.ai_interface.completed_actions[-1]["action_type"]
                    == interface.action_type.start_incantation
                    and self.ai_interface.completed_actions[-1]["success"]
                ):
                    self.state = WinnerAgent.State.none
                    self.need_for_more_players = 0
                else:
                    self.ai_interface.send_broadcast_action_and_block_until_response(f"wawa_{self.ai_interface.level}")

            elif self.state == WinnerAgent.State.passive_elevating:
                self.ai_interface.send_inventory_action_and_block_until_response()
                if (
                    self.ai_interface.last_known_inventory
                    and self.ai_interface.last_known_inventory[interface.object_type.food]
                    < self.low_food_elevation_abort_threshold
                ):  # critically low on food abort, also means the elavation did not succeed
                    self.state = WinnerAgent.State.none
                    self.need_for_more_players += 1

                if self.elevation_broadcast_direction is not None:
                    if self.elevation_broadcast_direction == interface.direction.none:
                        self.ai_interface.send_start_incantation_action_and_block_until_response()
                    else:
                        self.ai_interface.move_towards_absolute_direction(self.elevation_broadcast_direction)

                if (
                    self.ai_interface.completed_actions
                    and self.ai_interface.completed_actions[-1]["action_type"]
                    == interface.action_type.start_incantation
                    and self.ai_interface.completed_actions[-1]["success"]
                ):
                    self.state = WinnerAgent.State.none
                    self.need_for_more_players = 0
                else:
                    self.ai_interface.send_look_action_and_block_until_response()  # waiting

            self.elevation_broadcast_direction = None
