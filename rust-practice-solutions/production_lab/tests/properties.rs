use production_lab::Telemetry;
use proptest::prelude::*;

proptest! {
    #[test]
    fn arbitrary_bytes_do_not_panic(bytes in prop::collection::vec(any::<u8>(), 0..4096)) {
        let _ = serde_json::from_slice::<Telemetry>(&bytes);
    }

    #[test]
    fn finite_values_pass_validation(value in -1_000_000.0f64..1_000_000.0f64) {
        let telemetry = Telemetry {
            device_id: "d".into(),
            message_id: "m".into(),
            value,
            unit: "C".into(),
        };
        prop_assert!(telemetry.validate().is_ok());
    }
}
