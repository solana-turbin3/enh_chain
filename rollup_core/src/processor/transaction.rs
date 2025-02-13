use {
    serde::de, solana_sdk::{
        instruction::Instruction as SolanaInstruction, pubkey::Pubkey, signature::Keypair, system_instruction, transaction::{
            SanitizedTransaction as SolanaSanitizedTransaction, Transaction as SolanaTransaction,
        }
    }, spl_associated_token_account::get_associated_token_address, std::collections::HashSet
};

#[derive(Clone,Debug)]
pub enum TransactionMetadata {
    Transfer {
         mint: Option<Pubkey>,
         from: Pubkey,
         to: Pubkey,
         amount: u64,
    },
    CloseAccount {
        account : Pubkey,
        destination : Pubkey,
        owner : Pubkey,
    }    
}

impl From<&TransactionMetadata> for SolanaInstruction {
    fn from(value: &TransactionMetadata) -> Self {
        match value {
            &TransactionMetadata::Transfer { mint, from, to, amount } => {
                if let Some(mint) = mint {
                    let source_pubkey = get_associated_token_address(&from, &mint);
                    let destination_pubkey = get_associated_token_address(&to, &mint);
                    spl_token::instruction::transfer(
                        &spl_token::id(),
                        &source_pubkey,
                        &destination_pubkey,
                        &from,
                        &[],
                        amount,
                    )
                    .unwrap()
                } else {
                    system_instruction::transfer(&from, &to, amount)
                }
            },
            TransactionMetadata::CloseAccount { account, destination, owner} => {
                let account   = get_associated_token_address(&owner, &spl_token::native_mint::id());
                    spl_token::instruction::close_account(
                        &spl_token::id(),
                         &account,
                         destination,
                           owner,
                            &[],
                            )
                            .unwrap()
            }
    }
}
}

impl From<&TransactionMetadata> for SolanaTransaction {
    fn from(value: &TransactionMetadata) -> Self {
        match value {
            TransactionMetadata::Transfer { mint, from, to, amount } => {
                SolanaTransaction::new_with_payer(&[SolanaInstruction::from(value)], Some(from))
            },
            TransactionMetadata::CloseAccount { account, destination, owner } => {
                SolanaTransaction::new_with_payer(&[SolanaInstruction::from(value)], Some(owner))
            }
        }
    }
}

impl From<&TransactionMetadata> for SolanaSanitizedTransaction {
    fn from(value: &TransactionMetadata) -> Self {
        SolanaSanitizedTransaction::try_from_legacy_transaction(
            SolanaTransaction::from(value),
            &HashSet::new(),
        )
        .unwrap()
    }
}

/// Create a batch of Solana transactions, for the Solana SVM's transaction
/// processor, from a batch of PayTube instructions.
pub fn create_svm_transactions(
    paytube_transactions: &[TransactionMetadata],
) -> Vec<SolanaSanitizedTransaction> {
    paytube_transactions
        .iter()
        .map(SolanaSanitizedTransaction::from)
        .collect()
}

