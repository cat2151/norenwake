use crate::git_ops::{
    build_remote_safety_lines, build_target_origin_push_url, build_target_origin_url,
};

#[test]
fn remote_safety_flags_dangerous_origin() {
    let source = "https://github.com/me/source.git";
    let expected = build_target_origin_url("me", "my-new-repo");
    let expected_push = build_target_origin_push_url("me", "my-new-repo");
    let lines = build_remote_safety_lines(
        source,
        Some(source),
        Some(source),
        Some(source),
        &expected,
        &expected_push,
    );
    assert!(lines
        .iter()
        .any(|l| l.contains("NG: origin が暖簾分け元を向いています")));
}
