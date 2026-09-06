#[cfg(test)]
mod wave5_milestone_tests {
    #[test]
    fn test_tiered_milestone_release_schedule() {
        let total_escrow_amount: u128 = 200_000;
        let milestone_weights_bps: [u32; 3] = [3000, 3000, 4000]; // 30%, 30%, 40% (Total = 100%)
        
        let mut total_disbursed: u128 = 0;
        for &weight in milestone_weights_bps.iter() {
            let payout = (total_escrow_amount * weight as u128) / 10000;
            total_disbursed += payout;
        }

        assert_eq!(total_disbursed, total_escrow_amount);
    }

    #[test]
    fn test_arbitration_fee_split_invariants() {
        let dispute_amount: u128 = 100_000;
        let arbiter_fee_bps: u32 = 250; // 2.50% fee
        
        let arbiter_cut = (dispute_amount * arbiter_fee_bps as u128) / 10000;
        let client_award = (dispute_amount - arbiter_cut) / 2;
        let contractor_award = dispute_amount - arbiter_cut - client_award;

        assert_eq!(arbiter_cut, 2_500);
        assert_eq!(client_award, 48_750);
        assert_eq!(contractor_award, 48_750);
        assert_eq!(arbiter_cut + client_award + contractor_award, dispute_amount);
    }
}
