# Creator Staking Pallet

The Subsocial Creator Staking Pallet is a critical component of the Subsocial content creation ecosystem. It provides features for tracking staking information for creators by era, storing general info about total staking amounts per era, and managing the economic interactions between creators and their supporters.

## Features

- **Staking Information**: The pallet tracks staking information for creators by era, allowing users to stake funds to their favorite creators.

- **Unstaking and Withdrawal**: Users can start the unbonding process, and once the unbonding period has passed, they can withdraw their staked funds.

- **Reward Distribution**: The pallet calculates and distributes rewards for stakeholders and creators on a periodic basis (e.g., per era), incentivizing participation in the ecosystem.

- **Lock Periods**: To prevent immediate withdrawals, the pallet manages lock periods. Stakers must wait out an unbonding period after initiating an unstaking operation.

- **Events**: The pallet emits events for various lifecycle events, such as staking, unstaking, claiming rewards, and era changes, which can be used for monitoring and analytics.

- **Custom RPCs**: The pallet provides custom RPCs for querying staking information.

## License

This Subsocial Staking Pallet is open-source software released under the [GNU GPL3 License](LICENSE). Feel free to use, modify, and distribute it as needed.

## Acknowledgements

Special thanks for [Astar Network](https://github.com/AstarNetwork) for their [Pallet dapps-staking](https://github.com/AstarNetwork/astar-frame/tree/polkadot-v0.9.39/frame/dapps-staking).

## Support and Contact

If you have any questions or need support, please don't hesitate to [open an issue](https://github.com/dappforce/subsocial-parachain/issues).
