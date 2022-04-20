# Domains Pallet

Domains pallet allows users to register and manage their domains. The main purpose of a domain is to be a human-readable alias for owner's account address. 


## Main storages

The main storages of domains pallet are: 
- `RegisteredDomains: Map<DomainName,DomainMeta>`\
  Metadata associated per domain.
- `DomainsByOwner: Map<AccountId, DomainName[]>`\
  Domains owned per account.


## Domain metadata

In addition to resolving a domain owner, you can also resolve the various records set up
by the user. The following table shows a list of records that can be attached to the domain name.


<table>
<thead>
  <tr>
    <th>Record Name</th>
    <th>Type</th>
  </tr>
</thead>
<tbody>
  <tr>
    <td rowspan="3">Content</td>
    <td>String</td>
  </tr>
  <tr>
    <td>IPFS</td>
  </tr>
  <tr>
    <td>Hypercore</td>
  </tr>
  <tr>
    <td rowspan="3">InnerValue</td>
    <td>Subsocial Account</td>
  </tr>
  <tr>
    <td>Subsocial Space</td>
  </tr>
  <tr>
    <td>Subsocial Post</td>
  </tr>
  <tr>
    <td>OuterValue</td>
    <td>String</td>
  </tr>
</tbody>
</table>

### Main types

```rust
pub struct DomainMeta<T: Config> {
    /// When the domain was created.
    created: WhoAndWhenOf<T>,
    /// When the domain was updated.
    updated: Option<WhoAndWhenOf<T>>,

    /// Specific block, when the domain will become unavailable.
    expires_at: T::BlockNumber,

    /// The domain owner.
    owner: T::AccountId,

    /// Some additional domain metadata. For example avatar and description for this domain.
    content: Content,

    /// The inner domain link to Subsocial entity such as Account, Space, or Post.
    inner_value: Option<InnerValueOf<T>>,

    /// The outer domain link (any string).
    outer_value: Option<OuterValue<T>>,

    /// The amount was held as a deposit for storing this structure.
    domain_deposit: BalanceOf<T>,
    /// The amount was held as a deposit for storing outer value.
    outer_value_deposit: BalanceOf<T>,
}

pub struct WhoAndWhen {
    account: AccountId,
    block: BlockNumber,
    time: Moment,
}

pub enum InnerValue {
    Account(AccountId),
    Space(SpaceId),
    Post(PostId),
}
```

## JS examples

Let's see how to get data about domains and their owners from the storage of domains pallet.

### Get the domain owner

In many scenarios we may want to resolve a domain name
to the owner of this domain. This can be done with the following script.

```javascript
async function fetchDomainOwner(domain) {
    const domainMeta = await api.query.domains.registeredDomains(domain);
    return domainMeta.unwrap().owner;
}
```

### Get domains owned by account

```javascript
async function fetchDomains(account) {
    const domains = await api.query.domains.domainsByOwner(account);
    return domains;
}
```

### Get all TLDs (top-level domains)

The domains pallet stores a list of supported TLDs. This list can be fetched with
the following script.

```javascript
async function supportedTlds()  {
    const tldEntries = await api.query.domains.supportedTlds.entries();
    return tldEntries
        .filter(([_, isSupported]) => isSupported)
        .map(([key, _]) => key.args[0].toHuman());
}
```
