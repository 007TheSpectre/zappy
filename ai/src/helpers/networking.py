import socket


class AI_server_connection:
    def __init__(self, host: str, port: int):
        self.host = host
        self.port = port
        self.socket = None
        self.buffer = ""

    def connect(self) -> bool:
        try:
            self.socket = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
            self.socket.connect((self.host, self.port))
            return True
        except Exception:
            return False

    def is_connected(self) -> bool:
        return self.socket is not None

    def send_message(self, message: str) -> bool:
        if self.socket is None:
            return False
        try:
            self.socket.sendall(message.encode())
            return True
        except Exception:
            return False

    def blocking_receive(self) -> bool:
        if self.socket is None:
            return False
        try:
            while "\n" not in self.buffer:
                self.socket.setblocking(True)
                data = self.socket.recv(1024)
                if not data:
                    return False
                self.buffer += data.decode()
            return True
        except Exception:
            return False

    def nonblocking_receive(self) -> bool:
        if self.socket is None:
            return False
        try:
            self.socket.setblocking(False)
            while "\n" not in self.buffer:
                try:
                    data = self.socket.recv(1024)
                    if not data:
                        return False
                    self.buffer += data.decode()
                except BlockingIOError:
                    break
            return True
        except Exception:
            return False

    def is_message_available(self) -> bool:
        return "\n" in self.buffer

    def get_message(self) -> str | None:
        if "\n" in self.buffer:
            line, self.buffer = self.buffer.split("\n", 1)
            return line + "\n"
        return None

    def close(self):
        if self.socket is not None:
            self.socket.close()
            self.socket = None

    def __del__(self):
        self.close()
