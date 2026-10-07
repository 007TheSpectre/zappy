import helpers.interface as interface
from enum import Enum
from random import random, randint
from helpers.can_cast import can_cast
from subprocess import Popen
from os import getcwd, environ
import sys


cwd = getcwd()
DEBUG = environ.get("ZAPPY_DEBUG") is not None


def dbg(*args):
    if DEBUG:
        print(*args, file=sys.stderr, flush=True)


class WinnerAgent2:
    class State(Enum):
        none = 0
        collecting_ressources = 1
        egg_laying = 2
        active_elevating = 3
        passive_elevating = 4

    def __init__(self, ai_interface: interface.AI_server_interface):
        self.ai_interface = ai_interface
        self.state = WinnerAgent2.State.none
        self.elevation_ressources_dropped = False
        self.elevation_broadcast_direction = None
        # How many more loop iterations a heard rendezvous beacon stays valid.
        # Persisting it (instead of clearing every iteration) lets a player keep
        # homing toward an active elevator across several steps and stops it from
        # spuriously becoming a competing anchor — which is what stalled level 4+.
        self.beacon_ttl = 0
        # A level-agnostic homing pull toward ANY heard rendezvous beacon. Keeps
        # the whole swarm physically clustered while collecting so that whenever
        # players cross a level threshold there are >=4 of them co-located to ride
        # the next incantation up together — this is what breaks the level-4 funnel
        # (4->5 needs 4 same-level players on one tile, which never happened while
        # players collected resources by wandering off individually).
        self.rally_direction = None
        self.rally_ttl = 0
        # Single-leader election. Every player carries a fixed random priority and
        # broadcasts it inside its beacon ("wawa_<level>_<priority>"). A player that
        # hears a same-level beacon LOUDER (higher priority) than its own yields and
        # homes toward it, so all same-level players converge on ONE tile (the global
        # max priority) instead of splitting across many competing anchors — which is
        # exactly why 12 level-4 players never got 4 onto a single tile.
        self.anchor_priority = randint(1, 10**9)
        self.heard_leader_priority = 0
        # When our own level changes (we got elevated, possibly by another player's
        # incantation while we just stood on the tile), drop all rendezvous state and
        # re-plan from scratch for the new level.
        self.last_seen_level = 1
        self.need_for_more_players = 0

        # Thresholds tuned to the food level players actually sustain (~16 at f=1000):
        # the old 28 activation floor was ABOVE equilibrium, so anchors never formed
        # and nothing ever elevated. An incantation only costs ~2.4 food (1 food per
        # 126 time-units), so committing at ~14 food is safe; waiting passives also
        # eat off the rally tile.
        self.low_food_elevation_abort_threshold = 10
        self.minimum_food_for_active_elevation = 16
        self.minimum_food_for_passive_elevation = 14
        self.minimum_food_for_egg_laying = 18

    def run(self):
        while True:
            if not self.ai_interface.is_connected():
                print("Connection to the server lost.")
                break
            self.ai_interface.handle_server_messages_nonblocking()
            if not self.ai_interface.is_alive():
                print("Trantorian is no longer alive.")
                break

            if self.ai_interface.level != self.last_seen_level:
                self.last_seen_level = self.ai_interface.level
                self.state = WinnerAgent2.State.none
                self.elevation_broadcast_direction = None
                self.beacon_ttl = 0
                self.heard_leader_priority = 0
                self.need_for_more_players = 0

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

            # Drain the inbox into a local snapshot and clear it in one shot.
            # Iterating the live list while calling .remove() on it skips every
            # other element, so when several beacons land in the same tick some
            # were silently lost — starving the rally the whole strategy relies on.
            pending_messages = self.ai_interface.message_history
            self.ai_interface.message_history = []
            for message in pending_messages:
                if message["text"].startswith("wawa_"):
                    # "wawa_<level>" (legacy) or "wawa_<level>_<priority>".
                    suffix = message["text"][5:].split("_")
                    beacon_level = int(suffix[0]) if can_cast(str, int, suffix[0]) else None
                    beacon_priority = int(suffix[1]) if len(suffix) > 1 and can_cast(str, int, suffix[1]) else 0
                    if beacon_level is not None:
                        # Any beacon (regardless of level) drives swarm clustering.
                        self.rally_direction = message["direction"]
                        self.rally_ttl = 10
                        # Only follow a same-level beacon that is at least as loud as
                        # the loudest leader we've heard recently — this makes every
                        # player converge on the single global-max-priority anchor.
                        dbg(
                            f"[L{self.ai_interface.level}] heard beacon lvl={beacon_level} prio={beacon_priority} dir={message['direction']}"
                        )
                        if beacon_level == self.ai_interface.level and beacon_priority >= self.heard_leader_priority:
                            self.heard_leader_priority = beacon_priority
                            # A same-tile beacon (direction.none) means "incantate here";
                            # don't let a later directional beacon pull us off the tile.
                            if self.elevation_broadcast_direction != interface.direction.none:
                                self.elevation_broadcast_direction = message["direction"]
                            # Longer persistence so the elected leader's beacon doesn't
                            # expire between broadcasts (that flapping let lower-priority
                            # rivals re-activate and split the group instead of converging).
                            self.beacon_ttl = 8

            # True when a same-level anchor louder than us is currently rallying:
            # we must yield and home toward it rather than start a rival anchor.
            heard_louder_leader = self.beacon_ttl > 0 and self.heard_leader_priority > self.anchor_priority

            if self.state == WinnerAgent2.State.none:
                self.ai_interface.send_inventory_action()
                self.ai_interface.send_connect_nbr_action_and_block_until_response()
                # Elevation takes priority over egg-laying: a ready player must
                # become/join an anchor rather than fork (forking starved anchor
                # formation and diluted the cohort with level-1 newbies).
                if (
                    not heard_louder_leader  # we are the loudest known anchor for our level
                    and self.ai_interface.has_enough_ressources_for_next_level()
                    and self.ai_interface.last_known_inventory is not None
                    and self.ai_interface.last_known_inventory[interface.object_type.food]
                    >= self.minimum_food_for_active_elevation
                ):
                    self.elevation_ressources_dropped = False
                    self.state = WinnerAgent2.State.active_elevating
                elif (
                    heard_louder_leader  # a louder leader is rallying our level — go join it
                    and self.ai_interface.last_known_inventory is not None
                    and self.ai_interface.last_known_inventory[interface.object_type.food]
                    >= self.minimum_food_for_passive_elevation
                ):
                    self.state = WinnerAgent2.State.passive_elevating
                elif (
                    self.ai_interface.last_known_number_of_unused_slots is not None
                    and self.ai_interface.last_known_number_of_unused_slots == 0
                    and self.ai_interface.last_known_inventory is not None
                    and self.ai_interface.last_known_inventory[interface.object_type.food]
                    >= self.minimum_food_for_egg_laying
                ):
                    self.state = WinnerAgent2.State.egg_laying
                else:
                    self.state = WinnerAgent2.State.collecting_ressources
                inv = self.ai_interface.last_known_inventory
                food = inv[interface.object_type.food] if inv else "?"
                dbg(
                    f"[L{self.ai_interface.level}] -> {self.state.name} food={food} "
                    f"has_res={self.ai_interface.has_enough_ressources_for_next_level()} "
                    f"louder={heard_louder_leader} prio={self.anchor_priority}"
                )

            if self.state == WinnerAgent2.State.collecting_ressources:
                if self.rally_direction is not None:
                    # A rendezvous is active: drift toward it (or stay put if already
                    # on the tile) so the swarm stays dense and same-level players
                    # accumulate together for the next multi-player incantation.
                    if self.rally_direction != interface.direction.none:
                        self.ai_interface.move_towards_absolute_direction(self.rally_direction)
                else:
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
                self.state = WinnerAgent2.State.none

            elif self.state == WinnerAgent2.State.egg_laying:
                self.ai_interface.send_fork_action_and_block_until_response()
                self.state = WinnerAgent2.State.none

            elif self.state == WinnerAgent2.State.active_elevating:
                if heard_louder_leader:
                    # A louder same-level anchor showed up: stop being a rival anchor
                    # and go converge on it so the players actually pile onto one tile.
                    self.state = WinnerAgent2.State.passive_elevating
                    continue
                self.ai_interface.send_inventory_action_and_block_until_response()
                inventory = self.ai_interface.last_known_inventory
                if inventory and inventory[interface.object_type.food] < self.low_food_elevation_abort_threshold:
                    # critically low on food: abort and go feed/collect instead of
                    # burning a guaranteed-to-fail incantation
                    self.state = WinnerAgent2.State.none
                    self.need_for_more_players += 1
                else:
                    # The incantation consumes stones from the GROUND, so read the
                    # tile with a fresh Look (the previous bug read a stale Inventory
                    # and always saw 0) and top the ground up from our inventory.
                    look = self.ai_interface.send_look_action_and_block_until_response()
                    ground = (
                        look["response"][0]
                        if look
                        and look["action_type"] == interface.action_type.look
                        and look["success"]
                        and look["response"]
                        else {}
                    )
                    # Eat any food on our tile: the anchor stays put broadcasting and
                    # waiting for the crowd, so it must keep feeding or it starves
                    # before enough players arrive (this crashed the population).
                    for _ in range(ground.get(interface.object_type.food, 0)):
                        self.ai_interface.send_take_action_and_block_until_response(interface.object_type.food)
                    ready = True
                    for resource, required_amount in interface.RESSOURSE_REQUIREMENTS_FOR_LEVEL_UP.get(
                        self.ai_interface.level, {}
                    ).items():
                        on_ground = ground.get(resource, 0)
                        if on_ground < required_amount:
                            in_inventory = inventory.get(resource, 0) if inventory else 0
                            to_drop = min(required_amount - on_ground, in_inventory)
                            for _ in range(to_drop):
                                self.ai_interface.send_set_object_down_action_and_block_until_response(resource)
                            if on_ground + to_drop < required_amount:
                                ready = False  # not enough stones on hand yet
                    if not ready:
                        # missing stones: go collect more rather than spin on ko
                        dbg(f"[L{self.ai_interface.level}] active but ground not ready -> collect; ground={ground}")
                        self.elevation_ressources_dropped = False
                        self.state = WinnerAgent2.State.collecting_ressources
                    else:
                        self.ai_interface.send_start_incantation_action_and_block_until_response()
                        if (
                            self.ai_interface.completed_actions
                            and self.ai_interface.completed_actions[-1]["action_type"]
                            == interface.action_type.start_incantation
                            and self.ai_interface.completed_actions[-1]["success"]
                        ):
                            self.state = WinnerAgent2.State.none
                            self.need_for_more_players = 0
                        else:
                            dbg(
                                f"[L{self.ai_interface.level}] ANCHOR cast failed -> broadcast wawa_{self.ai_interface.level}_{self.anchor_priority}"
                            )
                            self.ai_interface.send_broadcast_action_and_block_until_response(
                                f"wawa_{self.ai_interface.level}_{self.anchor_priority}"
                            )

            elif self.state == WinnerAgent2.State.passive_elevating:
                self.ai_interface.send_inventory_action_and_block_until_response()
                inventory = self.ai_interface.last_known_inventory
                if inventory and inventory[interface.object_type.food] < self.low_food_elevation_abort_threshold:
                    # critically low on food: abort and go feed/collect
                    self.state = WinnerAgent2.State.none
                    self.need_for_more_players += 1
                elif self.elevation_broadcast_direction == interface.direction.none:
                    # We are on the leader's tile. First EAT any food on this tile so we
                    # don't starve while waiting (this was killing the rendezvous: idle
                    # waiters drained food and died before 4 ever gathered). Then drop
                    # every needed stone we carry into the shared pool (over-supply is
                    # harmless) and wait — the leader triggers the cast, which elevates
                    # everyone of our level on the tile. We don't self-cast: a premature
                    # cast with too few players burns 300 time-units.
                    look = self.ai_interface.send_look_action_and_block_until_response()
                    ground = (
                        look["response"][0]
                        if look
                        and look["action_type"] == interface.action_type.look
                        and look["success"]
                        and look["response"]
                        else {}
                    )
                    for _ in range(ground.get(interface.object_type.food, 0)):
                        self.ai_interface.send_take_action(interface.object_type.food)
                    for resource in interface.RESSOURSE_REQUIREMENTS_FOR_LEVEL_UP.get(self.ai_interface.level, {}):
                        for _ in range(inventory.get(resource, 0) if inventory else 0):
                            self.ai_interface.send_set_object_down_action(resource)
                    self.ai_interface.block_until_all_responses_received()
                elif self.elevation_broadcast_direction is not None:
                    dbg(f"[L{self.ai_interface.level}] passive homing dir={self.elevation_broadcast_direction}")
                    self.ai_interface.move_towards_absolute_direction(self.elevation_broadcast_direction)
                else:
                    # beacon expired — the leader is gone, re-plan
                    self.state = WinnerAgent2.State.none

            # Let a heard beacon persist for a few iterations so homing survives
            # gaps between broadcasts, instead of being wiped every loop.
            if self.beacon_ttl > 0:
                self.beacon_ttl -= 1
            if self.beacon_ttl == 0:
                self.elevation_broadcast_direction = None
                self.heard_leader_priority = 0

            if self.rally_ttl > 0:
                self.rally_ttl -= 1
            if self.rally_ttl == 0:
                self.rally_direction = None
