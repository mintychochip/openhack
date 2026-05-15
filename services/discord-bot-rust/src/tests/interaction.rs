use crate::interaction::{verify_signature, InteractionContext};

#[test]
fn verify_signature_rejects_invalid_public_key() {
    let result = verify_signature("aabbccdd", "timestamp", b"body", "invalid_hex");
    assert!(!result);
}

#[test]
fn verify_signature_rejects_malformed_hex() {
    let result = verify_signature("not_hex_at_all!", "also_bad", b"body", "ts");
    assert!(!result);
}

#[test]
fn verify_signature_rejects_wrong_signature() {
    let valid_pk = "0000000000000000000000000000000000000000000000000000000000000000";
    let result = verify_signature("aabbccdd", "timestamp", b"body", valid_pk);
    assert!(!result);
}

#[test]
fn interaction_context_fields_are_correct() {
    let ctx = InteractionContext {
        id: "interaction_123".into(),
        token: "token_abc".into(),
        guild_id: Some("guild_789".into()),
        channel_id: Some("channel_456".into()),
        discord_user_id: "discord_user_456".into(),
        data: Some(serde_json::json!({"type": 2})),
    };
    assert_eq!(ctx.id, "interaction_123");
    assert_eq!(ctx.token, "token_abc");
    assert_eq!(ctx.guild_id, Some("guild_789".to_string()));
    assert_eq!(ctx.channel_id, Some("channel_456".to_string()));
    assert_eq!(ctx.discord_user_id, "discord_user_456");
    assert!(ctx.data.is_some());
}
