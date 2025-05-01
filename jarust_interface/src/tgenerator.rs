use std::fmt::Debug;
use std::ops::Deref;

/// GenerateTransaction can be provided to an interface for generating messages transactions.
///
/// The more they're unique the better, especially when choosing with WebSockets interface as demultiplexing heavily relies on transaction ids.
pub trait GenerateTransaction: Send + Sync + 'static {
    fn generate_transaction(&self) -> String;
}

impl<F> GenerateTransaction for F
where
    F: Fn() -> String + Send + Sync + 'static,
{
    fn generate_transaction(&self) -> String {
        self()
    }
}

impl Debug for dyn GenerateTransaction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("GenerateTransaction").finish()
    }
}

#[derive(Debug)]
pub struct TransactionGenerator(Box<dyn GenerateTransaction>);

impl TransactionGenerator {
    pub fn new(generator: impl GenerateTransaction) -> Self {
        Self(Box::new(generator))
    }
}

impl Deref for TransactionGenerator {
    type Target = Box<dyn GenerateTransaction>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Debug)]
pub struct RandomTransactionGenerator;

impl GenerateTransaction for RandomTransactionGenerator {
    fn generate_transaction(&self) -> String {
        use rand::distributions::Alphanumeric;
        use rand::Rng;

        rand::thread_rng()
            .sample_iter(&Alphanumeric)
            .take(12)
            .map(char::from)
            .collect()
    }
}

#[derive(Debug)]
pub struct UuidTransactionGenerator;

impl GenerateTransaction for UuidTransactionGenerator {
    fn generate_transaction(&self) -> String {
        uuid::Uuid::new_v4().to_string()
    }
}
