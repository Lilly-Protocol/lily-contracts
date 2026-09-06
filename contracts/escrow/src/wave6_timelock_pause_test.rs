#[cfg(test)]
mod wave6_timelock_tests {
    #[test]
    fn test_escrow_timelock_challenge_window() {
        let submission_timestamp: u64 = 1_788_500_000;
        let challenge_period_seconds: u64 = 86_400 * 2; // 48 hours
        
        let unlock_timestamp = submission_timestamp + challenge_period_seconds;
        
        let can_release = |current_ts: u64| -> bool {
            current_ts >= unlock_timestamp
        };

        assert!(!can_release(submission_timestamp + 3600)); // 1 hr after: cannot release
        assert!(!can_release(unlock_timestamp - 1)); // 1 sec before: cannot release
        assert!(can_release(unlock_timestamp)); // At unlock: can release
        assert!(can_release(unlock_timestamp + 86400)); // After unlock: can release
    }

    #[test]
    fn test_emergency_pause_state_guard() {
        struct EscrowState {
            is_paused: bool,
            balance: u128,
        }

        let execute_disbursement = |state: &EscrowState, amount: u128| -> Result<u128, &'static str> {
            if state.is_paused {
                return Err("Contract is paused");
            }
            if amount > state.balance {
                return Err("Insufficient balance");
            }
            Ok(state.balance - amount)
        };

        let normal_state = EscrowState { is_paused: false, balance: 50_000 };
        assert_eq!(execute_disbursement(&normal_state, 10_000), Ok(40_000));

        let paused_state = EscrowState { is_paused: true, balance: 50_000 };
        assert_eq!(execute_disbursement(&paused_state, 10_000), Err("Contract is paused"));
    }
}
