#![allow(dead_code)]
#![allow(unused_imports)]
use types::{PublicKey, URef, U256};

type TokenId = String;
type URI = String;

trait WithStorage<Storage: CEP47Storage> {
    fn storage(&self) -> Storage;
}

trait CEP47Contract<Storage: CEP47Storage>: WithStorage<Storage> {
    // Metadata
    fn name(&self) -> String {
        self.storage().name()
    }

    fn symbol(&self) -> String {
        self.storage().symbol()
    }

    fn uri(&self) -> URI {
        self.storage().uri()
    }

    // Getters
    fn balance_of(&self, owner: PublicKey) -> U256 {
        self.storage().balance_of(owner)
    }

    fn owner_of(&self, token_id: TokenId) -> Option<PublicKey> {
        self.storage().onwer_of(token_id)
    }

    fn total_supply(&self) -> U256 {
        self.storage().total_supply()
    }

    fn token_uri(&self, token_id: TokenId) -> Option<URI> {
        self.storage().token_uri(token_id)
    }

    fn tokens(&self, owner: PublicKey) -> Vec<TokenId> {
        self.storage().get_tokens(owner)
    }

    // Minter function.
    // Guarded by the entrypoint group.
    fn mint_one(&mut self, recipient: PublicKey, token_uri: URI) {
        self.storage()
            .mint_copies(recipient, token_uri, U256::one());
    }

    fn mint_many(&mut self, recipient: PublicKey, token_uris: Vec<URI>) {
        self.storage().mint_many(recipient, token_uris);
    }

    fn mint_copies(&mut self, recipient: PublicKey, token_uri: URI, count: U256) {
        self.storage().mint_copies(recipient, token_uri, count);
    }

    // Transfer functions.
    fn transfer_token(&mut self, sender: PublicKey, recipient: PublicKey, token_id: TokenId) {
        // 1. Load tokens owned by the sender.
        let mut sender_tokens = self.storage().get_tokens(sender);
        // 2. Assert that token_id is in sender_tokens.
        assert!(
            sender_tokens.contains(&token_id),
            "wrong owner of token {}",
            token_id
        );
        // 3. Remove token_id from sender_tokens.
        sender_tokens.retain(|x| x.clone() != token_id);
        self.storage().set_tokens(sender, sender_tokens);

        // 4. Add token_id to the recipient tokens
        let mut recipient_tokens = self.storage().get_tokens(recipient);
        recipient_tokens.push(token_id);
        self.storage().set_tokens(recipient, recipient_tokens);
    }

    fn transfer_many_tokens(
        &mut self,
        sender: PublicKey,
        recipient: PublicKey,
        token_ids: Vec<TokenId>,
    ) {
        let mut sender_tokens = self.storage().get_tokens(sender);
        for token_id in token_ids.iter() {
            assert!(sender_tokens.contains(token_id), "wrong token {}", token_id);
            sender_tokens.retain(|x| x.clone() != token_id.clone());
        }
        let mut recipient_tokens = self.storage().get_tokens(recipient);
        recipient_tokens.append(&mut token_ids.clone());
        self.storage().set_tokens(sender, sender_tokens);
        self.storage().set_tokens(recipient, recipient_tokens);
    }

    fn transfer_all_tokens(&mut self, sender: PublicKey, recipient: PublicKey) {
        let mut sender_tokens = self.storage().get_tokens(sender);
        let mut recipient_tokens = self.storage().get_tokens(recipient);
        recipient_tokens.append(&mut sender_tokens);

        self.storage().set_tokens(sender, sender_tokens);
        self.storage().set_tokens(recipient, recipient_tokens);
    }

    // URef releated function.
    fn detach(&mut self, owner: PublicKey, token_id: TokenId) -> URef {
        todo!();
    }
    fn attach(&mut self, token_uref: URef, recipient: PublicKey) {}
    fn token_id(&self, token_uref: URef) -> TokenId {
        todo!();
    }
}

trait CEP47Storage {
    // Metadata.
    fn name(&self) -> String;
    fn symbol(&self) -> String;
    fn uri(&self) -> URI;

    // Getters
    fn balance_of(&self, owner: PublicKey) -> U256;
    fn onwer_of(&self, token_id: TokenId) -> Option<PublicKey>;
    fn total_supply(&self) -> U256;
    fn token_uri(&self, token_id: TokenId) -> Option<URI>;

    // Setters
    fn get_tokens(&self, owner: PublicKey) -> Vec<TokenId>;
    fn set_tokens(&mut self, owner: PublicKey, token_ids: Vec<TokenId>);
    fn mint_many(&mut self, recipient: PublicKey, token_uris: Vec<URI>);
    fn mint_copies(&mut self, recipient: PublicKey, token_uri: URI, count: U256);
    fn new_uref(&mut self, token_id: TokenId) -> URef;
    fn del_uref(&mut self, token_uref: URef);
}

#[cfg(test)]
mod tests {
    use super::{CEP47Contract, CEP47Storage, WithStorage};
    use super::{PublicKey, TokenId, U256, URI};
    use std::{
        collections::{hash_map::DefaultHasher, BTreeMap},
        hash::{Hash, Hasher},
    };

    struct TestStorage {
        name: String,
        symbol: String,
        uri: URI,
        total_supply: U256,
        tokens: BTreeMap<PublicKey, Vec<TokenId>>,
        token_uris: BTreeMap<TokenId, URI>,
        balances: BTreeMap<PublicKey, U256>,
        belongs_to: BTreeMap<TokenId, PublicKey>,
    }

    impl TestStorage {
        pub fn new() -> TestStorage {
            TestStorage {
                name: String::from("Casper Enhancement Proposal 47"),
                symbol: String::from("CEP47"),
                uri: URI::from("https://github.com/casper-ecosystem/casper-nft-cep47"),
                total_supply: U256::from(0),
                tokens: BTreeMap::new(),
                balances: BTreeMap::new(),
                belongs_to: BTreeMap::new(),
                token_uris: BTreeMap::new(),
            }
        }
    }

    impl CEP47Storage for TestStorage {
        fn name(&self) -> String {
            self.name.clone()
        }

        fn symbol(&self) -> String {
            self.symbol.clone()
        }

        fn uri(&self) -> URI {
            self.uri.clone()
        }

        fn balance_of(&self, owner: PublicKey) -> U256 {
            self.balances.get(&owner).unwrap().clone()
        }

        fn onwer_of(&self, token_id: TokenId) -> Option<PublicKey> {
            let owner = self.belongs_to.get(&token_id);
            if owner.is_some() {
                Some(owner.unwrap().clone())
            } else {
                None
            }
        }

        fn total_supply(&self) -> U256 {
            self.total_supply
        }

        fn token_uri(&self, token_id: TokenId) -> Option<URI> {
            let uri = self.token_uris.get(&token_id);
            if uri.is_some() {
                Some(uri.unwrap().clone())
            } else {
                None
            }
        }

        fn get_tokens(&self, owner: PublicKey) -> Vec<TokenId> {
            self.tokens.get(&owner).unwrap().clone()
        }

        fn set_tokens(&mut self, owner: PublicKey, token_ids: Vec<TokenId>) {
            let owner_balance = self.balances.get(&owner);
            let mut owner_new_balance = if owner_balance.is_none() {
                U256::from(0)
            } else {
                owner_balance.unwrap().clone()
            };

            for token_id in token_ids.clone() {
                let prev_owner = self.belongs_to.get(&token_id).unwrap().clone();
                let prev_owner_balance = self.balances.get(&prev_owner).unwrap().clone();
                self.balances.insert(prev_owner, prev_owner_balance - 1);
                self.belongs_to.insert(token_id, owner);
                owner_new_balance = owner_new_balance + 1;
                self.balances.insert(owner, owner_new_balance);
            }
        }

        fn mint_many(&mut self, recipient: PublicKey, token_uris: Vec<URI>) {
            let recipient_balance = self.balances.get(&recipient);
            let recipient_tokens = self.tokens.get(&recipient);
            let mut recipient_new_balance = if recipient_balance.is_none() {
                U256::from(0)
            } else {
                recipient_balance.unwrap().clone()
            };
            let mut recipient_new_tokens = if recipient_tokens.is_none() {
                Vec::<TokenId>::new()
            } else {
                recipient_tokens.unwrap().clone()
            };

            let mut hasher = DefaultHasher::new();

            for token_uri in token_uris.clone() {
                let token_info = (self.total_supply, self.uri.clone(), token_uri.clone());
                Hash::hash(&token_info, &mut hasher);
                let token_id: TokenId = TokenId::from(hasher.finish().to_string());
                self.token_uris.insert(token_id.clone(), token_uri);
                recipient_new_tokens.push(token_id.clone());
                self.belongs_to.insert(token_id, recipient);
                recipient_new_balance = recipient_new_balance + 1;
                self.total_supply = self.total_supply + 1;
            }
            self.balances.insert(recipient, recipient_new_balance);
            self.tokens.insert(recipient, recipient_new_tokens);
        }

        fn mint_copies(&mut self, recipient: PublicKey, token_uri: URI, count: U256) {
            let token_uris: Vec<URI> = vec![token_uri; count.as_usize()];
            self.mint_many(recipient, token_uris);
        }

        fn new_uref(&mut self, token_id: super::TokenId) -> types::URef {
            todo!()
        }

        fn del_uref(&mut self, token_uref: types::URef) {
            todo!()
        }
    }

    struct TestContract {}

    impl WithStorage<TestStorage> for TestContract {
        fn storage(&self) -> TestStorage {
            TestStorage::new()
        }
    }

    impl CEP47Contract<TestStorage> for TestContract {}

    #[test]
    fn test_metadata() {
        let contract = TestContract {};
        assert_eq!(
            contract.name(),
            String::from("Casper Enhancement Proposal 47")
        );
        assert_eq!(contract.symbol(), String::from("CEP47"));
        assert_eq!(
            contract.uri(),
            String::from("https://github.com/casper-ecosystem/casper-nft-cep47")
        );
    }
}