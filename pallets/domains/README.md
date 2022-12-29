# Domains Pallet

Domains pallet allows users to register and manage their domains. The main purpose of a domain is to be a human-readable alias for owner's account address. 


## Main storages

The main storages of domains pallet are: 
- `RegisteredDomains: Map<DomainName,DomainMeta>`\
  Metadata associated with a domain.
- `DomainsByOwner: Map<AccountId, DomainName[]>`\
  Domains owned by an account.
- `DomainRecords: Map<DomainName, Map<RecordKey, RecordValueWithDeposit>>`\
  All records associated with a given domain name.


## Domain Records

In addition to resolving a domain owner, you can also resolve the various records set up
by the user.

The domains record system allows any dotsama domain to have a key-value pairs of metadata attached to it.
There is no restrictions on what value should be stored in what key, however we have a list of defined key schema to be 
interpreted by clients.


### Record Key standard

A standard record key is split by namespaces by a . used as a separator.

#### Crypto payment records

A key of records regarding payment in crypto follows one of the following formats

* `crypto.<TICKER>.address`
  * `crypto.ETH.address` ⇒ `0xD1E5b0FF1287aA9f9A268759062E4Ab08b9Dacbe`
  * `crypto.BTC.address` ⇒ `bc1qkd4um2nn2uyzmsch5y86wsa2pfh8xl445lg9nv`
* `crypto.<TICKER>.version.<VERSION>.address`
  * `crypto.USDT.version.ERC20.address` ⇒ `0x8aaD44321A86b170879d7A244c1e8d360c99DdA8`
  * `crypto.USDT.version.TRON.address` ⇒ `THG9jVSMfKEbg4vYTYWjmLRyga3CKZdDsk`

#### Browser resolution

* `browser.redirect_url` ⇒ `http://example.com/home.html`

#### Social records

* `social.picture.value` ⇒ `ipfs://QmQqzMTavQgT4f4T5v6PWBp7XNKtoPmC9jvn12WPT3gkSE`
* `whois.email.value` ⇒ `tarekkma@gmail.com`
* `whois.for_sale.value` ⇒ `false`
* `social.twitter.username` ⇒ `TarekkMA1`
* `social.subsocial.account` ⇒ `3tiKakuy6RaHWThsHts23De8XwpaNdvw8PwwXcJqeVSN6w8w`
* `social.subsocial.space` ⇒ `15`
* `social.subsocial.post` ⇒ `1`


### Main types

* `DomainName`: array of characters representing a domain name (maximum length `Config::MaxDomainLength`)
* `RecordKey`: array of characters representing record key (maximum length `Config::MaxRecordKeySize`)
* `RecordValue`: array of characters representing record value (maximum length `Config::MaxRecordValueSize`)

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

    /// The amount was held as a deposit for storing this structure.
    domain_deposit: BalanceOf<T>,
}

pub struct WhoAndWhen {
    account: AccountId,
    block: BlockNumber,
    time: Moment,
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
