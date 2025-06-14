# TODO list (Found at|Done at| Description)

- [ ] [954ffc2](https://github.com/mkamonmdt/learn_coin/commit/954ffc26e38611ad57fc77fb2a36f769cdec77cb)|| Opcodes are fun, can we make them binary-compatible?
- [ ] [954ffc2](https://github.com/mkamonmdt/learn_coin/commit/954ffc26e38611ad57fc77fb2a36f769cdec77cb)|| Contract interface should be external to BChain, maybe trait?
- [ ] [954ffc2](https://github.com/mkamonmdt/learn_coin/commit/954ffc26e38611ad57fc77fb2a36f769cdec77cb)||  Nasty u64->2xu32 pointer handling
- [ ] [5600a9c](https://github.com/mkamonmdt/learn_coin/commit/5600a9cc0fb592f28ae2a38488849b7d272c9ef2)|| Implement slashing mechanism 
- [ ] [50d88b9](https://github.com/mkamonmdt/learn_coin/commit/50d88b94a100bd8e5db414b2191951020a162d57)|| comparision between floats should be done involving epsilon
- [ ] [50d88b9](https://github.com/mkamonmdt/learn_coin/commit/50d88b94a100bd8e5db414b2191951020a162d57)|| magic number 2 for validator consensus should be named
- [ ] [50d88b9](https://github.com/mkamonmdt/learn_coin/commit/50d88b94a100bd8e5db414b2191951020a162d57)|| no tests for validator selection functionality
- [ ] [50d88b9](https://github.com/mkamonmdt/learn_coin/commit/50d88b94a100bd8e5db414b2191951020a162d57)|| no tests for unstaking functionality
- [ ] [50d88b9](https://github.com/mkamonmdt/learn_coin/commit/50d88b94a100bd8e5db414b2191951020a162d57)|| transaction error handling: what should be done with invalid block?


# Solved list:

- [x] [8244cad](https://github.com/mkamonmdt/learn_coin/commit/8244cade376cbd176ba06bb9b111d41b51375a3c)|[fdcafd9](https://github.com/mkamonmdt/learn_coin/commit/fdcafd90ed61d2131e9d908339aefef0b2000b31)| There is no need to traverse entire blockchain history every epoch to calculate validators. More efficient approach might be to keep track of relevant stake_pool, and update it epoch-by-epoch.
- [x] [421b491](https://github.com/mkamonmdt/learn_coin/commit/421b491ca376872b7bd20425e0dfc849ffb6cd1a)|[50d88b9](https://github.com/mkamonmdt/learn_coin/commit/50d88b94a100bd8e5db414b2191951020a162d57)| once stake is put, it stays there indefinitely. The users cannot withdraw. Mostlikely stakes should be put on specific Epoch(s), returned after some time.
- [x] [8244cad](https://github.com/mkamonmdt/learn_coin/commit/8244cade376cbd176ba06bb9b111d41b51375a3c)|[6f393ee](https://github.com/mkamonmdt/learn_coin/commit/6f393ee6c76c6296780be8c994a1fa2de1ff5e1a)| Validators are still selected with O(n^2) complexity as in each slot we traverse almost entire blockchain. We should calculate validators once after adding last block of epoch N-2 and store them localy for ongoing operation purposes.
- [-] [5600a9c](https://github.com/mkamonmdt/learn_coin/commit/5600a9cc0fb592f28ae2a38488849b7d272c9ef2)|invalidated| Validators for epoch N once caclulated should posted during epoch N-1 and easily accessible for futher logic implementation
- [x] [42cc793](https://github.com/mkamonMdt/learn_coin/commit/42cc7937cafaf89e22b74035437207ad31c62276)|current| a validator might want to have information of being selected for a slot in advance. Currently we calculate a validator based on most recent block.
- [-] [63b3541](https://github.com/mkamonMdt/learn_coin/commit/63b3541b25a00e5d8b09ce7bec9ed66bc80a788a)|invalidated| 1:1 mapping between block slot and a validator: with greater number of stakers than slots, some will never get picked
- [x] [63b3541](https://github.com/mkamonMdt/learn_coin/commit/63b3541b25a00e5d8b09ce7bec9ed66bc80a788a)|[421b491](https://github.com/mkamonMdt/learn_coin/commit/421b491ca376872b7bd20425e0dfc849ffb6cd1a)| stake_pool can be modified during current epoch, i.e. when seed of RNG is known 
- [x] [63b3541](https://github.com/mkamonMdt/learn_coin/commit/63b3541b25a00e5d8b09ce7bec9ed66bc80a788a)|[421b491](https://github.com/mkamonMdt/learn_coin/commit/421b491ca376872b7bd20425e0dfc849ffb6cd1a)| stake_pool can differ from each peer perspective, i.e. there is no stake_pool consensus mechanism
- [x] [63b3541](https://github.com/mkamonMdt/learn_coin/commit/63b3541b25a00e5d8b09ce7bec9ed66bc80a788a)|[42cc793](https://github.com/mkamonMdt/learn_coin/commit/42cc7937cafaf89e22b74035437207ad31c62276)| 1:1 mapping between block slot and a validator: With larger epoch size that might not be feasible to calculate a validator for each slot and store the hash.
- [x] [63b3541](https://github.com/mkamonMdt/learn_coin/commit/63b3541b25a00e5d8b09ce7bec9ed66bc80a788a)|[42cc793](https://github.com/mkamonMdt/learn_coin/commit/42cc7937cafaf89e22b74035437207ad31c62276)| 1:1 mapping between block slot and a validator: it does not result it fair proportionallity.
- [x] [63b3541](https://github.com/mkamonMdt/learn_coin/commit/63b3541b25a00e5d8b09ce7bec9ed66bc80a788a)|[42cc793](https://github.com/mkamonMdt/learn_coin/commit/42cc7937cafaf89e22b74035437207ad31c62276)| 1:1 mapping between block slot and a validator: with stakers less than slots, the pool will be prematurely exhausted
- [x] [63b3541](https://github.com/mkamonMdt/learn_coin/commit/63b3541b25a00e5d8b09ce7bec9ed66bc80a788a)|[42cc793](https://github.com/mkamonMdt/learn_coin/commit/42cc7937cafaf89e22b74035437207ad31c62276)| the validator_to_slot_assignment vector is fully calculated repetitevely for each slot
