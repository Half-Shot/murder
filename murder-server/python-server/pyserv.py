#!/usr/bin/python3
from murder_client import MurderClient


def main():
    client = MurderClient()
    client.connect("127.0.0.1", 1337)

    for i in range(1, 10):
        client.add_player("Foo bar")
    client.advance() # Morning
    client.advance() # Special
    detective = [x for x in client.players() if x.role == "Detective"]
    client.investigate_target()
    client.vote_kill(0, 1)

    client.close()

if __name__ == "__main__":
    main()
