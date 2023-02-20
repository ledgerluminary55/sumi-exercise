# Exercise

Please create a pallet which tracks offchain events submitted by an oracle. You should set aside around 1-2 hours for this exercise.

An event consists of some bytes of some unknown size. The event can only be submitted by the oracle.
The pallet trusts an oracle, and allows setting the account which is considered to be the current oracle.
The pallet discards events which were recieved more than an hour ago.
Please keep in mind any security and quality considerations that would normally be taken for such a task including tests and benchmarks. If you are running out of time, please note your thoughts for any remaining work in the pallet.

# Solution

## Implementation

The pallet has the following features:

### Storage:

- The pallet has a storage item which stores the current oracle account. (This could be a map if we want to support multiple oracles)

- The pallet has a storage item which is a bounded vec of events. Right now the size of the bounded vec is 1000, but this could be changed to a different value that fits the requirements.

When a event is removed because it is no longer valid, the vector is reorganized and because of that the index changes. This is a problem to have a consistent index to access the event data. We could implement a circular buffer to solve this problem. Instead of erasing the event, we could just mark it as invalid and keep the index consistent.

These events have the following fields:

- The event data: This is a bounded vec of bytes.

- The oracle account: This is the account that submitted the event.

- The timestamp: This is the block number when the event was submitted.

### Extrinsics:

- Add oracle: This extrinsic allows the root to set the current oracle account. This should definitely be expanded to allow multiple oracles. We could use a map to store if a certain account is an oracle.
  It is assumed that the root account is the one that will have the power to control some of the pallet's parameters.
  As a good practice, it is expected that the root account is not just a single account, but a set of accounts that can be managed by a multisig, a DAO, a council, or any other mechanism that allows for a group of people to have control over the pallet.

- Submit event: This extrinsic allows the current oracle to submit an event.

### Hooks:

- On_idle: This hook is used to remove events that are older than an hour.
  The implementation is just a simplification to show the idea. Here we should definitely check the remaining weight and remove as many events as possible. If we don't have enough weight to remove all the events, we should just remove the oldest ones. In case we don't have enough weight to remove any event, we should just skip the removal. If we do not perform this check the entire chain could be bricked.
