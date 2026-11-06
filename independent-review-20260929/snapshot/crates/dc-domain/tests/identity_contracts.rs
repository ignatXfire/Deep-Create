use dc_domain::ids::EntityId;

#[test]
fn identity_rejects_wrong_uuid_version_and_variant() {
    let valid = 0x017f_22e2_79b0_7cc3_98c4_dc0c_0c07_398f_u128;
    assert!(EntityId::from_u128(valid).is_some());
    for version in 0..16_u128 {
        let candidate = (valid & !(0xf << 76)) | (version << 76);
        assert_eq!(EntityId::from_u128(candidate).is_some(), version == 7);
    }
    for variant in 0..4_u128 {
        let candidate = (valid & !(0x3 << 62)) | (variant << 62);
        assert_eq!(EntityId::from_u128(candidate).is_some(), variant == 2);
    }
    assert!(EntityId::from_u128(0).is_none());
    assert!(EntityId::from_u128(u128::MAX).is_none());
}

#[test]
fn uuidv7_wire_parser_cannot_bypass_binary_validation() {
    for input in [
        "ffffffffffffffffffffffffffffffff",
        "017f22e279b04cc398c4dc0c0c07398f",
        "017f22e279b07cc3d8c4dc0c0c07398f",
    ] {
        assert!(serde_json::from_str::<EntityId>(&format!("\"{input}\"")).is_err());
    }
    let input = "\"017f22e279b07cc398c4dc0c0c07398f\"";
    let id: EntityId = serde_json::from_str(input).unwrap();
    assert_eq!(serde_json::to_string(&id).unwrap(), input);
}
