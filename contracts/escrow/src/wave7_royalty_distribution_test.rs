#[cfg(test)]
mod wave7_royalty_tests {
    #[test]
    fn test_royalty_bps_distribution() {
        let gross_amount: u128 = 100_000;
        let platform_fee_bps: u128 = 250; // 2.50%
        let creator_royalty_bps: u128 = 500; // 5.00%
        let bps_divisor: u128 = 10_000;

        let calculate_share = |amount: u128, bps: u128| -> u128 {
            (amount * bps) / bps_divisor
        };

        let platform_fee = calculate_share(gross_amount, platform_fee_bps);
        let creator_royalty = calculate_share(gross_amount, creator_royalty_bps);
        let net_payout = gross_amount - platform_fee - creator_royalty;

        assert_eq!(platform_fee, 2_500);
        assert_eq!(creator_royalty, 5_000);
        assert_eq!(net_payout, 92_500);
        assert_eq!(platform_fee + creator_royalty + net_payout, gross_amount);
    }

    #[test]
    fn test_minimum_disbursement_dust_guard() {
        let min_disbursement: u128 = 500;
        let is_valid_disbursement = |amount: u128| -> bool {
            amount >= min_disbursement
        };

        assert!(is_valid_disbursement(10_000));
        assert!(is_valid_disbursement(500));
        assert!(!is_valid_disbursement(499));
        assert!(!is_valid_disbursement(0));
    }
}
