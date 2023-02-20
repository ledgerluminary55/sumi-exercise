# Exercise

Please create a pallet which tracks offchain events submitted by an oracle. You should set aside around 1-2 hours for this exercise.

An event consists of some bytes of some unknown size. The event can only be submitted by the oracle.
The pallet trusts an oracle, and allows setting the account which is considered to be the current oracle.
The pallet discards events which were recieved more than an hour ago.
Please keep in mind any security and quality considerations that would normally be taken for such a task including tests and benchmarks. If you are running out of time, please note your thoughts for any remaining work in the pallet.
