# TODO list (Found at|Done at| Description)

- [ ] [421b491](https://github.com/mkamonMdt/learn_coin/commit/421b491ca376872b7bd20425e0dfc849ffb6cd1a)|| once stake is put, it stays there indefinitely. The users cannot withdraw. Mostlikely stakes should be put on specific Epoch(s), returned after some time.
- [ ] [current]()|| Implement slashing mechanism 
- [ ] [current]()|| Validators for epoch N once caclulated should posted during epoch N-1 and easily accessible for futher logic implementation

# Solved list:

- [x] [42cc793](https://github.com/mkamonMdt/learn_coin/commit/42cc7937cafaf89e22b74035437207ad31c62276)|current| a validator might want to have information of being selected for a slot in advance. Currently we calculate a validator based on most recent block.
- [-] [63b3541](https://github.com/mkamonMdt/learn_coin/commit/63b3541b25a00e5d8b09ce7bec9ed66bc80a788a)|invalidated| 1:1 mapping between block slot and a validator: with greater number of stakers than slots, some will never get picked
- [x] [63b3541](https://github.com/mkamonMdt/learn_coin/commit/63b3541b25a00e5d8b09ce7bec9ed66bc80a788a)|[421b491](https://github.com/mkamonMdt/learn_coin/commit/421b491ca376872b7bd20425e0dfc849ffb6cd1a)| stake_pool can be modified during current epoch, i.e. when seed of RNG is known 
- [x] [63b3541](https://github.com/mkamonMdt/learn_coin/commit/63b3541b25a00e5d8b09ce7bec9ed66bc80a788a)|[421b491](https://github.com/mkamonMdt/learn_coin/commit/421b491ca376872b7bd20425e0dfc849ffb6cd1a)| stake_pool can differ from each peer perspective, i.e. there is no stake_pool consensus mechanism
- [x] [63b3541](https://github.com/mkamonMdt/learn_coin/commit/63b3541b25a00e5d8b09ce7bec9ed66bc80a788a)|[42cc793](https://github.com/mkamonMdt/learn_coin/commit/42cc7937cafaf89e22b74035437207ad31c62276)| 1:1 mapping between block slot and a validator: With larger epoch size that might not be feasible to calculate a validator for each slot and store the hash.
- [x] [63b3541](https://github.com/mkamonMdt/learn_coin/commit/63b3541b25a00e5d8b09ce7bec9ed66bc80a788a)|[42cc793](https://github.com/mkamonMdt/learn_coin/commit/42cc7937cafaf89e22b74035437207ad31c62276)| 1:1 mapping between block slot and a validator: it does not result it fair proportionallity.
- [x] [63b3541](https://github.com/mkamonMdt/learn_coin/commit/63b3541b25a00e5d8b09ce7bec9ed66bc80a788a)|[42cc793](https://github.com/mkamonMdt/learn_coin/commit/42cc7937cafaf89e22b74035437207ad31c62276)| 1:1 mapping between block slot and a validator: with stakers less than slots, the pool will be prematurely exhausted
- [x] [63b3541](https://github.com/mkamonMdt/learn_coin/commit/63b3541b25a00e5d8b09ce7bec9ed66bc80a788a)|[42cc793](https://github.com/mkamonMdt/learn_coin/commit/42cc7937cafaf89e22b74035437207ad31c62276)| the validator_to_slot_assignment vector is fully calculated repetitevely for each slot
