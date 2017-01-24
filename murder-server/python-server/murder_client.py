import socket
import murder_pb2
import copy
from murder_exception import MurderError
SUPPORTED_VER = 0x00


class MurderClient:

    def __init__(self):
        self.__socket = socket.socket(socket.AF_INET, socket.SOCK_STREAM)

    def connect(self, host, port):
        self.__socket.connect((host, port))
        # Make a new game
        self.__socket.sendall(self.build_packet(1, b""))
        packet = self.__get_packet()
        self.uuid = packet.uuid
        self.__players = {}

    def add_player(self, name):
        proto = murder_pb2.CliAddPlayer()
        proto.player_name = name
        tx = self.build_packet(2, proto.SerializeToString())
        self.__socket.sendall(tx)
        id = self.__get_packet().player_id
        self.__players[id] = {
            "name": name,
            "role": None
        }

    def players(self):
        return copy.deepcopy(self.__players)

    def advance(self):
        self.__socket.sendall(self.build_packet(4, b""))
        pkt = self.__get_packet()
        if type(pkt) == murder_pb2.SrvRoles:
            for i in range(0, len(pkt.role)):
                self.__players[i]["role"] = pkt.role[i]

    def vote_kill(self, player, victim):
        proto = murder_pb2.CliVote()
        proto.sender = player
        proto.victim = victim
        self.__socket.sendall(self.build_packet(7, proto.SerializeToString()))
        pkt = self.__get_packet()
        return pkt.votes

    def close(self):
        self.__socket.close()

    def __get_packet(self):
        header = self.__socket.recv(10)
        length = int.from_bytes(header[6:10], byteorder='big')
        payload = self.__socket.recv(length)
        if header[5] == 1:
            proto = murder_pb2.SrvNewGame()
            # New game response
        elif header[5] == 2:
            proto = murder_pb2.SrvAddPlayer()
            # New player response
        elif header[5] == 5:
            proto = murder_pb2.SrvState()
        elif header[5] == 6:
            proto = murder_pb2.SrvRoles()
        elif header[5] == 7:
            proto = murder_pb2.SrvVote()
        elif header[5] == 200:
            return None
            # Unknown request
        elif header[5] == 201:
            # error
            proto = murder_pb2.SrvError()
            proto.ParseFromString(payload)
            raise MurderError(proto)
        else:
            raise Exception("Unknown packet type: " + str(header[5]))
        proto.ParseFromString(payload)
        return proto

    def build_packet(self, request_type, payload, version=0x00):
        data = b""
        data += bytes([0x25, 0xC9, 0xC3, 0x5F])
        data += bytes([version])
        data += bytes([request_type])
        data += len(payload).to_bytes(4, byteorder='big')
        data += payload
        data += bytes(0x00)
        return data


def print_packet(buffer):
    print(' '.join(format(x, '02x') for x in buffer))
