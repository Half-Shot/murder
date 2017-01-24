# Murder Server API Guide

## Packet Structure

```
magic string (4B)                  : 25 C9 C3 5F
client supported version (1B)      : 00
request type (1B)                  : 00
payload length (4B)                : 00 00 AB 9A = 65000
payload (xB)                       :
```

## Connecting

Connecting doesn't require you to send any data, but the server will drop any connections with no activity after a set timeout.
Please send a heartbeat regularly to avoid a dropped connection.

## Request Types

## ``00`` Heartbeat
To check if the server exists, you may connect and send a ``00`` type
with an empty payload. The server should respond with the same information but with a single byte of value ``01`` in the payload.

## NewGame - 01
Client | Server  
FirstCall

## AddPlayer - 02
Client | Server  
Selection

## RemovePlayer - 03
Client | Server  
Selection

## Advance - 04
Client  
All

## State - 05
Client | Server  
All

## Roles - 06
Server  
Morning | Special | Mafia

## Vote - 07
Client | Server  
Morning | Special | Mafia

## ChatChannels - 08
Client | Server  
All

## DetectiveInvestigate - 09
Client | Server
Special

## Unknown Request - 200
Server

## Invalid Request - 201
Server
