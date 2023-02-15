# Subsocial Proxy
This pallet provides a simple wrapper around the proxy
pallet, it aims to give users first proxy for free (without
reserving any SUB).

This pallet works using the following method:
- Users calls `SubsocialProxy::add_free_proxy(....)`.
- The pallet will check if the user have no proxy defined before.
- if so the pallet sets a temporary storage flag to be true, and calls
`Proxy::add_proxy(....)`.
- When proxy pallet tries to calculate deposit it will result to zero
- After proxy is added SubsocialProxy removes the flag so depoists can be calculated
correctly again.


SubsocialProxy takes the original `ProxyDepositBase` and `ProxyDepositFactor` 
configs, and provide you with `AdjustedProxyDepositBase` and `AdjustedProxyDepositBase`.

```rust
impl pallet_subsocial_proxy::Config for Runtime {
    type ProxyDepositBase = ProxyDepositBase;
    type ProxyDepositFactor = ProxyDepositFactor;
}

impl pallet_proxy::Config for Runtime {
    ...
    type ProxyDepositBase = pallet_subsocial_proxy::AdjustedProxyDepositBase<Runtime>;
    type ProxyDepositFactor = pallet_subsocial_proxy::AdjustedProxyDepositFactor<Runtime>;
    ...
}

```
