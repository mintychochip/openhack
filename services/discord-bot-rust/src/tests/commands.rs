use crate::commands::{
    embed_response, ephemeral_response, get_custom_id, get_int_option, get_string_option,
    make_embed,
};
use crate::interaction::InteractionContext;

fn test_ctx(data: Option<serde_json::Value>) -> InteractionContext {
    InteractionContext {
        id: "123".into(),
        token: "tok".into(),
        guild_id: None,
        channel_id: None,
        discord_user_id: "user1".into(),
        data,
    }
}

#[test]
fn ephemeral_response_has_correct_structure() {
    let resp = ephemeral_response("test message");
    assert_eq!(resp["type"], 4);
    assert_eq!(resp["data"]["content"], "test message");
    assert_eq!(resp["data"]["flags"], 64);
}

#[test]
fn ephemeral_response_with_empty_content() {
    let resp = ephemeral_response("");
    assert_eq!(resp["type"], 4);
    assert_eq!(resp["data"]["content"], "");
    assert_eq!(resp["data"]["flags"], 64);
}

#[test]
fn embed_response_has_correct_structure() {
    let embed = serde_json::json!({"title": "Test", "description": "Desc"});
    let resp = embed_response(embed.clone());
    assert_eq!(resp["type"], 4);
    assert_eq!(resp["data"]["embeds"][0]["title"], "Test");
    assert_eq!(resp["data"]["embeds"][0]["description"], "Desc");
}

#[test]
fn make_embed_has_correct_fields() {
    let embed = make_embed("Title", "Description", 0xFF_00_00);
    assert_eq!(embed["title"], "Title");
    assert_eq!(embed["description"], "Description");
    assert_eq!(embed["color"], 0xFF_00_00);
}

#[test]
fn get_string_option_extracts_value() {
    let ctx = test_ctx(Some(serde_json::json!({
        "options": [{
            "name": "team-create",
            "options": [
                {"name": "name", "value": "My Team"},
                {"name": "extra", "value": "extra_val"}
            ]
        }]
    })));
    assert_eq!(get_string_option(&ctx, "name"), Some("My Team".to_string()));
    assert_eq!(
        get_string_option(&ctx, "extra"),
        Some("extra_val".to_string())
    );
    assert_eq!(get_string_option(&ctx, "missing"), None);
}

#[test]
fn get_int_option_extracts_value() {
    let ctx = test_ctx(Some(serde_json::json!({
        "options": [{
            "name": "leaderboard",
            "options": [
                {"name": "top", "value": 15}
            ]
        }]
    })));
    assert_eq!(get_int_option(&ctx, "top"), Some(15));
    assert_eq!(get_int_option(&ctx, "missing"), None);
}

#[test]
fn get_string_option_returns_none_when_no_data() {
    let ctx = test_ctx(None);
    assert_eq!(get_string_option(&ctx, "name"), None);
}

#[test]
fn get_custom_id_extracts_from_component_data() {
    let ctx = test_ctx(Some(serde_json::json!({
        "custom_id": "openhack:join-team:abc123",
        "component_type": 2
    })));
    assert_eq!(get_custom_id(&ctx), "openhack:join-team:abc123");
}

#[test]
fn get_custom_id_returns_empty_when_no_data() {
    let ctx = test_ctx(None);
    assert_eq!(get_custom_id(&ctx), "");
}
