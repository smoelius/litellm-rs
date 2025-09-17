use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

/// Password reset token database model
#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize, Deserialize)]
#[sea_orm(table_name = "password_reset_tokens")]
pub struct Model {
    /// Token ID
    #[sea_orm(primary_key)]
    pub id: i32,

    /// User ID this token belongs to
    pub user_id: Uuid,

    /// Reset token (unique)
    #[sea_orm(unique)]
    pub token: String,

    /// Token expiration timestamp
    pub expires_at: DateTimeWithTimeZone,

    /// Token creation timestamp
    pub created_at: DateTimeWithTimeZone,

    /// Token usage timestamp (if used)
    pub used_at: Option<DateTimeWithTimeZone>,
}

/// Password reset token entity relations
#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    /// Belongs to user relation
    #[sea_orm(
        belongs_to = "super::user::Entity",
        from = "Column::UserId",
        to = "super::user::Column::Id"
    )]
    User,
}

impl Related<super::user::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::User.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
