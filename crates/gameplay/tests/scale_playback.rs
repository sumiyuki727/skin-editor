mod scale_support;
use scale_support::*;
use skin_editor_gameplay::*;

#[test]
fn g1_fixed_external_script_hits_ten_thousand_with_exact_streamed_golden_records() {
    let data = circles();
    let mut receipts = [None; 2];
    for (case, budget) in [WorkBudget::one(), WorkBudget::full()]
        .into_iter()
        .enumerate()
    {
        let mut play = Playback::new(&data, 300_001 + case as u128, budget);
        play.tail(0);
        receipts[case] = Some(play.verify_end(StatisticsOrigin::FromStart { start: t(START) }));
        assert_eq!(play.peaks.inputs, 64);
        assert_eq!(play.peaks.events, 64);
        assert_eq!(play.oracle.sequence, 20_001);
        assert_eq!(play.oracle.miss_count, 0);
        let c = SyntheticCoordinator::costs_for_objects(N).unwrap();
        assert_eq!(play.peaks.reserved_source, c.source_bytes);
        assert_eq!(
            play.peaks.reserved_runtime,
            c.pool_bytes + c.coordinator_bytes + c.run_bytes
        );
    }
    assert_eq!(receipts[0], receipts[1]);
}

#[test]
fn a1_actual_automatic_generator_matches_the_golden_semantics_under_both_work_budgets() {
    let data = circles();
    let mut receipts = [None; 2];
    for (case, budget) in [WorkBudget::one(), WorkBudget::full()]
        .into_iter()
        .enumerate()
    {
        let mut play = Playback::new(&data, 300_010 + case as u128, budget);
        play.automatic();
        play.tail(0);
        receipts[case] = Some(play.verify_end(StatisticsOrigin::FromStart { start: t(START) }));
        assert_eq!(play.oracle.sequence, 20_001);
        assert_eq!(play.peaks.inputs, 64);
        assert_eq!(play.peaks.events, 64);
        assert!(
            play.owner
                .current()
                .unwrap()
                .work_counts()
                .automatic_objects
                <= 2 * N as u64
        );
        let before = play.owner.current().unwrap().observation();
        let counts = play.owner.current().unwrap().work_counts();
        let allocation = play.owner.allocations();
        assert!(matches!(
            play.owner
                .prepare_automatic_inputs(play.source(), t(END), budget),
            Ok(AutomaticReport::Complete {
                steps: 0,
                admitted: 0,
                ..
            })
        ));
        assert_eq!(play.owner.current().unwrap().work_counts(), counts);
        assert_eq!(play.owner.current().unwrap().observation(), before);
        assert_eq!(play.owner.allocations(), allocation);
    }
    assert_eq!(receipts[0], receipts[1]);
}
