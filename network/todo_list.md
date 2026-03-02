# TODO list (Found at|Done at| Description)

- [ ] [33749eb](https://github.com/mkamonMdt/learn_coin/commit/33749eb4bb1503a5b57242631395a280a13f8eca) || implement id exchange. Currently each new pear will get a locally assigned uuid used for log purpose only. The uuid should be global so P2P network can identify peers network-wide
- [ ] [33749eb](https://github.com/mkamonMdt/learn_coin/commit/33749eb4bb1503a5b57242631395a280a13f8eca) || implement simple "say-hello" protocol. Since in BC a wallet public key can be used as UUID the protocol could establish TcpStream, each party would generate rand msg that should be singed by the other verifaiable with public key so we have a proof of possesion.

# Solved list:
