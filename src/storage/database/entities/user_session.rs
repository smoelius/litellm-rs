use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

/// User session database model
#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize, Deserialize)]
#[sea_orm(table_name = "user_sessions")]
pub struct Model {
    /// Session ID
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: String,

    /// User ID this session belongs to
    pub user_id: Uuid,

    /// Session expiration timestamp
    pub expires_at: DateTimeWithTimeZone,

    /// Session creation timestamp
    pub created_at: DateTimeWithTimeZone,

    /// Last access timestamp
    pub last_accessed_at: DateTimeWithTimeZone,

    /// Client IP address (optional)
    pub ip_address: Option<String>,

    /// Client user agent (optional)
    pub user_agent: Option<String>,

    /// Session active status
    pub is_active: bool,
}

/// User session entity relations
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
