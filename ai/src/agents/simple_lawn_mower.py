import helpers.interface as interface
from enum import Enum


class state(Enum):
    collection_looking = 1
    collection_picking = 2
    collection_moving = 3


class SimpleLawnMowerAgent:
    def __init__(self, ai_interface: interface.AI_server_interface):
        self.ai_interface = ai_interface
        self.state = state.collection_looking

    def run(self):
        awaiting_pickup_list = []
        invert_direction = False
        while True:
            if not self.ai_interface.is_connected():
                print("Connection to the server lost.")
                break
            self.ai_interface.handle_server_messages_nonblocking()
            if not self.ai_interface.is_alive():
                print("Trantorian is no longer alive.")
                break

            if self.state == state.collection_looking:
                self.ai_interface.send_look_action_and_block_until_response()
                self.state = state.collection_picking
            elif self.state == state.collection_picking:
                if awaiting_pickup_list == []:
                    for obj, count in self.ai_interface.completed_actions[-1]["response"][0].items():
                        if obj != interface.object_type.player and obj != interface.object_type.egg and count > 0:
                            for _ in range(count):
                                awaiting_pickup_list.append(obj)
                if awaiting_pickup_list != []:
                    self.ai_interface.send_take_action_and_block_until_response(
                        interface.str_to_object_type(awaiting_pickup_list[0])
                    )
                    awaiting_pickup_list.remove(awaiting_pickup_list[0])
                if awaiting_pickup_list == []:
                    self.state = state.collection_moving
            elif self.state == state.collection_moving:
                if (
                    self.ai_interface.relative_y_position
                    == max(self.ai_interface.world_x_size, self.ai_interface.world_y_size) // 2
                    and self.ai_interface.relative_x_position
                    == max(self.ai_interface.world_x_size, self.ai_interface.world_y_size) // 2
                    and self.ai_interface.relative_direction == interface.cardinal_direction.south
                ):  # pyright: ignore[reportArgumentType]
                    invert_direction = not invert_direction
                elif (
                    self.ai_interface.relative_y_position
                    == -max(self.ai_interface.world_x_size, self.ai_interface.world_y_size) // 2 + 1
                    and self.ai_interface.relative_x_position
                    == -max(self.ai_interface.world_x_size, self.ai_interface.world_y_size) // 2 + 1
                    and self.ai_interface.relative_direction == interface.cardinal_direction.north
                ):  # pyright: ignore[reportArgumentType]
                    invert_direction = not invert_direction
                if (
                    self.ai_interface.relative_x_position
                    == max(self.ai_interface.world_x_size, self.ai_interface.world_y_size) // 2
                    and self.ai_interface.relative_direction == interface.cardinal_direction.north
                ):  # pyright: ignore[reportArgumentType]
                    if invert_direction:
                        self.ai_interface.send_turn_left_action()
                        self.ai_interface.send_forward_action()
                        self.ai_interface.send_turn_left_action_and_block_until_response()
                    else:
                        self.ai_interface.send_turn_right_action()
                        self.ai_interface.send_forward_action()
                        self.ai_interface.send_turn_right_action_and_block_until_response()
                elif (
                    self.ai_interface.relative_x_position
                    == -max(self.ai_interface.world_x_size, self.ai_interface.world_y_size) // 2 + 1
                    and self.ai_interface.relative_direction == interface.cardinal_direction.south
                ):  # pyright: ignore[reportArgumentType]
                    if invert_direction:
                        self.ai_interface.send_turn_right_action()
                        self.ai_interface.send_forward_action()
                        self.ai_interface.send_turn_right_action_and_block_until_response()
                    else:
                        self.ai_interface.send_turn_left_action()
                        self.ai_interface.send_forward_action()
                        self.ai_interface.send_turn_left_action_and_block_until_response()
                else:
                    self.ai_interface.send_forward_action_and_block_until_response()

                self.state = state.collection_looking
