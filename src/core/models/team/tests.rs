//! Team module tests

use super::*;
use uuid::Uuid;

#[test]
fn test_team_creation() {
    let team = Team::new("test-team".to_string(), Some("Test Team".to_string()));

    assert_eq!(team.name, "test-team");
    assert_eq!(team.display_name, Some("Test Team".to_string()));
    assert!(team.is_active());
}

#[test]
fn test_team_usage_update() {
    let mut team = Team::new("test-team".to_string(), None);

    team.update_usage(10, 1000, 0.50);

    assert_eq!(team.usage_stats.total_requests, 10);
    assert_eq!(team.usage_stats.total_tokens, 1000);
    assert_eq!(team.usage_stats.total_cost, 0.50);
}

#[test]
fn test_team_member_permissions() {
    let team_id = Uuid::new_v4();
    let user_id = Uuid::new_v4();
    let mut member = TeamMember::new(team_id, user_id, TeamRole::Member, None);

    assert!(!member.has_permission("admin"));

    member.add_permission("admin".to_string());
    assert!(member.has_permission("admin"));

    member.remove_permission("admin");
    assert!(!member.has_permission("admin"));
}

#[test]
fn test_invitation_expiry() {
    let team_id = Uuid::new_v4();
    let invited_by = Uuid::new_v4();
    let expires_at = chrono::Utc::now() - chrono::Duration::hours(1); // Expired

    let invitation = TeamInvitation::new(
        team_id,
        "test@example.com".to_string(),
        TeamRole::Member,
        "token".to_string(),
        invited_by,
        expires_at,
    );

    assert!(invitation.is_expired());
}
