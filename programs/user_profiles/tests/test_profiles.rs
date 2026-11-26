use std::assert_eq;

use anchor_lang::{
    AccountDeserialize, InstructionData, Key, ToAccountMetas, prelude::Pubkey, solana_program::instruction::Instruction, system_program,
};
use litesvm::LiteSVM;
use solana_keypair::Keypair;
use solana_message::{Message, VersionedMessage};
use solana_signer::Signer;
use solana_transaction::{Transaction, versioned::VersionedTransaction};
use user_profiles::{UpdateProfile, UserProfile};

#[test]
fn test_initialize_profile() {
    let mut svm = LiteSVM::new();
    let program_id = user_profiles::ID;

    svm.add_program(
        program_id,
        include_bytes!("../../../target/deploy/user_profiles.so"),
    )
    .unwrap();

    let authority = Keypair::new();
    svm.airdrop(&authority.pubkey(), 1_000_000_000).unwrap();

    let (profile_pda, _bump) = Pubkey::find_program_address(
        &[b"profile", authority.pubkey().as_ref()],
        &program_id
    );

    let ix_data = user_profiles::instruction::InitializeProfile {
        name: "Hello".to_string(),
        bio: "World".to_string(),
    }.data();

    let accounts = user_profiles::accounts::InitializeProfile {
        user_profile: profile_pda,
        authority: authority.pubkey(),
        system_program: system_program::ID,
    }.to_account_metas(None);

    let ix = Instruction::new_with_bytes(program_id, &ix_data, accounts);

    let blockhash = svm.latest_blockhash();
    let msg = Message::new_with_blockhash(&[ix], Some(&authority.pubkey()), &blockhash);
    let tx = VersionedTransaction::try_new(VersionedMessage::Legacy(msg), &[&authority]).unwrap();

    svm.send_transaction(tx).unwrap();

    let account = svm.get_account(&profile_pda).expect("profile account not found");
    let profile = user_profiles::UserProfile::try_deserialize(&mut account.data.as_slice()).unwrap();

    assert_eq!(profile.name, "Hello");
    assert_eq!(profile.bio, "World");
    assert_eq!(profile.address, authority.pubkey());
}

#[test] 
fn test_update_profile() {
    let mut svm = LiteSVM::new();

    let program_id = user_profiles::ID;
    let program_bytes = include_bytes!("../../../target/deploy/user_profiles.so");

    svm.add_program(program_id, program_bytes).unwrap();

    let authority = Keypair::new();
    svm.airdrop(&authority.pubkey(), 1_000_000_000).unwrap();

    let (profile_pda, _bump) = Pubkey::find_program_address(&[b"profile", authority.pubkey().as_ref()], &program_id);

    let init_ix_data = user_profiles::instruction::InitializeProfile {
        name: "Hello".to_string(),
        bio: "World".to_string(),
    }.data();
    
    let init_accounts = user_profiles::accounts::InitializeProfile {
        user_profile: profile_pda,
        authority: authority.pubkey(),
        system_program: system_program::ID,
    }.to_account_metas(None);
    
    let init_ix = Instruction::new_with_bytes(program_id, &init_ix_data, init_accounts);
    
    let init_tx = Transaction::new_signed_with_payer(
        &[init_ix],
        Some(&authority.pubkey()),
        &[&authority],
        svm.latest_blockhash(),
    );

    svm.send_transaction(init_tx).unwrap();

    let update_ix = Instruction::new_with_bytes(
        program_id,
        &user_profiles::instruction::UpdateProfile {
            name: "Updated Name".to_string(),
            bio: "Updated Bio".to_string(),
        }.data(),
        user_profiles::accounts::UpdateProfile {
            authority: authority.pubkey(),
            user_profile: profile_pda,
        }.to_account_metas(None)
    );

        let update_tx = Transaction::new_signed_with_payer(
        &[update_ix],
        Some(&authority.pubkey()),
        &[&authority],
        svm.latest_blockhash(),
    );

    svm.send_transaction(update_tx).unwrap();

    let account = svm.get_account(&profile_pda).expect("profile not found");
    let profile = user_profiles::UserProfile::try_deserialize(&mut account.data.as_slice()).unwrap();

    assert_eq!(profile.name, "Updated Name");
    assert_eq!(profile.bio, "Updated Bio");
    assert_eq!(profile.address, authority.pubkey());

    let attacker = Keypair::new();
    svm.airdrop(&attacker.pubkey(), 1_000_000_000).unwrap();

    let attack_ix = Instruction::new_with_bytes(
        program_id,
        &user_profiles::instruction::UpdateProfile {
            name: "Hacked".to_string(),
            bio: "Hacked".to_string(),
        }.data(),
        user_profiles::accounts::UpdateProfile {
            authority: attacker.pubkey(),
            user_profile: profile_pda,
        }.to_account_metas(None),
    );

    let attack_tx = Transaction::new_signed_with_payer(
        &[attack_ix],
        Some(&attacker.pubkey()),
        &[&attacker],
        svm.latest_blockhash(),
    );

    let result = svm.send_transaction(attack_tx);
    let profile = user_profiles::UserProfile::try_deserialize(&mut account.data.as_slice()).unwrap();
    assert_eq!(profile.name, "Updated Name");
    assert_eq!(profile.bio, "Updated Bio");
}
