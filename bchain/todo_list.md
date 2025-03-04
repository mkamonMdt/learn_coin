# TODO list (Found|Done| Description)

- [ ] 63b3541|| 1:1 mapping between block slot and a validator: with greater number of stakers than slots, some will never get picked
- [ ] 63b3541|| stake_pool can be modified during current epoch, i.e. when seed of RNG is known 
- [ ] 63b3541|| stake_pool can differ from each peer perspective, i.e. there is no stake_pool consensus mechanism
- [ ] ce9371d|| a validator might want to have information of being selected for a slot in advance. Currently we calculate a validator based on most recent block.


# Solved list:

- [x] 63b3541|ce9371d| 1:1 mapping between block slot and a validator: With larger epoch size that might not be feasible to calculate a validator for each slot and store the hash.
- [x] 63b3541|ce9371d| 1:1 mapping between block slot and a validator: it does not result it fair proportionallity.
- [x] 63b3541|ce9371d| 1:1 mapping between block slot and a validator: with stakers less than slots, the pool will be prematurely exhausted
- [x] 63b3541|ce9371d| the validator_to_slot_assignment vector is fully calculated repetitevely for each slot
