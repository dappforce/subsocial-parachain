# Free Proxy

This pallet provides a simple wrapper around the proxy
pallet, it aims to give users first proxy for free (without
reserving any SUB).

This pallet works using the following method:
- Users calls `FreeProxy::add_free_proxy(....)`.
- The pallet will check if the user have no proxy defined before.
- if so the pallet sets a temporary storage flag so deposits are overridden to zero, and calls
`Proxy::add_proxy(....)`.
- When proxy pallet tries to calculate deposit it will result to zero
- After proxy is added FreeProxy removes the flag so depoists can be calculated
correctly again.


FreeProxy takes the original `ProxyDepositBase` and `ProxyDepositFactor` 
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

### Note!
- If the user tries to add a new proxy after the first "free" proxy, he will
have to reserve deposit for 2 proxies.
- And if the user tries to remove any proxy, he will be refunded deposits of only one proxy
- Other part of deposit will be refunded when the other proxy is also removed.

**So we can think of it as "pay later" proxy, not "free" proxy.** Since users will
reserve deposit for that "free" proxy when they add other proxy.