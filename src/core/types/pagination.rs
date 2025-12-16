//! Pagination and filtering types

use serde::{Deserialize, Serialize};

/// Pagination parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pagination {
    /// Page number (starts from 1)
    pub page: u32,
    /// Page size
    pub per_page: u32,
    /// Total count
    pub total: Option<u64>,
    /// Total pages
    pub total_pages: Option<u32>,
}

impl Default for Pagination {
    fn default() -> Self {
        Self {
            page: 1,
            per_page: 20,
            total: None,
            total_pages: None,
        }
    }
}

impl Pagination {
    /// Calculate offset
    pub fn offset(&self) -> u32 {
        (self.page - 1) * self.per_page
    }

    /// Set total and calculate total pages
    pub fn with_total(mut self, total: u64) -> Self {
        self.total = Some(total);
        self.total_pages = Some(((total as f64) / (self.per_page as f64)).ceil() as u32);
        self
    }
}

/// Sort parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SortOrder {
    /// Sort field
    pub field: String,
    /// Sort direction
    pub direction: SortDirection,
}

/// Sort direction
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SortDirection {
    /// Ascending
    #[default]
    Asc,
    /// Descending
    Desc,
}

/// Filter criteria
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Filter {
    /// Field name
    pub field: String,
    /// Operator
    pub operator: FilterOperator,
    /// Value
    pub value: serde_json::Value,
}

/// Filter operator
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FilterOperator {
    /// Equals
    Eq,
    /// Not equals
    Ne,
    /// Greater than
    Gt,
    /// Greater than or equal
    Gte,
    /// Less than
    Lt,
    /// Less than or equal
    Lte,
    /// Contains
    Contains,
    /// Not contains
    NotContains,
    /// In list
    In,
    /// Not in list
    NotIn,
    /// Starts with
    StartsWith,
    /// Ends with
    EndsWith,
    /// Regex match
    Regex,
    /// Is null
    IsNull,
    /// Is not null
    IsNotNull,
}
