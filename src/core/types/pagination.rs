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

#[cfg(test)]
mod tests {
    use super::*;

    // ==================== Pagination Tests ====================

    #[test]
    fn test_pagination_default() {
        let pagination = Pagination::default();
        assert_eq!(pagination.page, 1);
        assert_eq!(pagination.per_page, 20);
        assert!(pagination.total.is_none());
        assert!(pagination.total_pages.is_none());
    }

    #[test]
    fn test_pagination_offset_first_page() {
        let pagination = Pagination {
            page: 1,
            per_page: 20,
            total: None,
            total_pages: None,
        };
        assert_eq!(pagination.offset(), 0);
    }

    #[test]
    fn test_pagination_offset_second_page() {
        let pagination = Pagination {
            page: 2,
            per_page: 20,
            total: None,
            total_pages: None,
        };
        assert_eq!(pagination.offset(), 20);
    }

    #[test]
    fn test_pagination_offset_third_page() {
        let pagination = Pagination {
            page: 3,
            per_page: 10,
            total: None,
            total_pages: None,
        };
        assert_eq!(pagination.offset(), 20);
    }

    #[test]
    fn test_pagination_offset_custom_per_page() {
        let pagination = Pagination {
            page: 5,
            per_page: 50,
            total: None,
            total_pages: None,
        };
        assert_eq!(pagination.offset(), 200);
    }

    #[test]
    fn test_pagination_with_total_exact_pages() {
        let pagination = Pagination::default().with_total(100);
        assert_eq!(pagination.total, Some(100));
        assert_eq!(pagination.total_pages, Some(5)); // 100 / 20 = 5
    }

    #[test]
    fn test_pagination_with_total_partial_page() {
        let pagination = Pagination::default().with_total(101);
        assert_eq!(pagination.total, Some(101));
        assert_eq!(pagination.total_pages, Some(6)); // ceil(101 / 20) = 6
    }

    #[test]
    fn test_pagination_with_total_single_item() {
        let pagination = Pagination::default().with_total(1);
        assert_eq!(pagination.total, Some(1));
        assert_eq!(pagination.total_pages, Some(1));
    }

    #[test]
    fn test_pagination_with_total_zero() {
        let pagination = Pagination::default().with_total(0);
        assert_eq!(pagination.total, Some(0));
        assert_eq!(pagination.total_pages, Some(0));
    }

    #[test]
    fn test_pagination_clone() {
        let pagination = Pagination::default().with_total(100);
        let cloned = pagination.clone();
        assert_eq!(pagination.page, cloned.page);
        assert_eq!(pagination.total, cloned.total);
    }

    #[test]
    fn test_pagination_serialization() {
        let pagination = Pagination::default().with_total(50);
        let json = serde_json::to_value(&pagination).unwrap();
        assert_eq!(json["page"], 1);
        assert_eq!(json["per_page"], 20);
        assert_eq!(json["total"], 50);
    }

    #[test]
    fn test_pagination_deserialization() {
        let json = r#"{"page": 2, "per_page": 10, "total": 100, "total_pages": 10}"#;
        let pagination: Pagination = serde_json::from_str(json).unwrap();
        assert_eq!(pagination.page, 2);
        assert_eq!(pagination.per_page, 10);
        assert_eq!(pagination.total, Some(100));
    }

    // ==================== SortDirection Tests ====================

    #[test]
    fn test_sort_direction_default() {
        let direction = SortDirection::default();
        assert_eq!(direction, SortDirection::Asc);
    }

    #[test]
    fn test_sort_direction_asc_serialization() {
        let direction = SortDirection::Asc;
        let json = serde_json::to_string(&direction).unwrap();
        assert_eq!(json, "\"asc\"");
    }

    #[test]
    fn test_sort_direction_desc_serialization() {
        let direction = SortDirection::Desc;
        let json = serde_json::to_string(&direction).unwrap();
        assert_eq!(json, "\"desc\"");
    }

    #[test]
    fn test_sort_direction_deserialization() {
        let asc: SortDirection = serde_json::from_str("\"asc\"").unwrap();
        let desc: SortDirection = serde_json::from_str("\"desc\"").unwrap();
        assert_eq!(asc, SortDirection::Asc);
        assert_eq!(desc, SortDirection::Desc);
    }

    #[test]
    fn test_sort_direction_clone() {
        let direction = SortDirection::Desc;
        let cloned = direction.clone();
        assert_eq!(direction, cloned);
    }

    // ==================== SortOrder Tests ====================

    #[test]
    fn test_sort_order_structure() {
        let sort = SortOrder {
            field: "created_at".to_string(),
            direction: SortDirection::Desc,
        };
        assert_eq!(sort.field, "created_at");
        assert_eq!(sort.direction, SortDirection::Desc);
    }

    #[test]
    fn test_sort_order_serialization() {
        let sort = SortOrder {
            field: "name".to_string(),
            direction: SortDirection::Asc,
        };
        let json = serde_json::to_value(&sort).unwrap();
        assert_eq!(json["field"], "name");
        assert_eq!(json["direction"], "asc");
    }

    #[test]
    fn test_sort_order_deserialization() {
        let json = r#"{"field": "id", "direction": "desc"}"#;
        let sort: SortOrder = serde_json::from_str(json).unwrap();
        assert_eq!(sort.field, "id");
        assert_eq!(sort.direction, SortDirection::Desc);
    }

    #[test]
    fn test_sort_order_clone() {
        let sort = SortOrder {
            field: "test".to_string(),
            direction: SortDirection::Asc,
        };
        let cloned = sort.clone();
        assert_eq!(sort.field, cloned.field);
    }

    // ==================== Filter Tests ====================

    #[test]
    fn test_filter_structure() {
        let filter = Filter {
            field: "status".to_string(),
            operator: FilterOperator::Eq,
            value: serde_json::json!("active"),
        };
        assert_eq!(filter.field, "status");
        assert_eq!(filter.value, "active");
    }

    #[test]
    fn test_filter_with_numeric_value() {
        let filter = Filter {
            field: "count".to_string(),
            operator: FilterOperator::Gt,
            value: serde_json::json!(10),
        };
        assert_eq!(filter.value, 10);
    }

    #[test]
    fn test_filter_serialization() {
        let filter = Filter {
            field: "name".to_string(),
            operator: FilterOperator::Contains,
            value: serde_json::json!("test"),
        };
        let json = serde_json::to_value(&filter).unwrap();
        assert_eq!(json["field"], "name");
        assert_eq!(json["operator"], "contains");
    }

    #[test]
    fn test_filter_clone() {
        let filter = Filter {
            field: "test".to_string(),
            operator: FilterOperator::Eq,
            value: serde_json::json!("value"),
        };
        let cloned = filter.clone();
        assert_eq!(filter.field, cloned.field);
    }

    // ==================== FilterOperator Tests ====================

    #[test]
    fn test_filter_operator_eq_serialization() {
        let op = FilterOperator::Eq;
        let json = serde_json::to_string(&op).unwrap();
        assert_eq!(json, "\"eq\"");
    }

    #[test]
    fn test_filter_operator_ne_serialization() {
        let op = FilterOperator::Ne;
        let json = serde_json::to_string(&op).unwrap();
        assert_eq!(json, "\"ne\"");
    }

    #[test]
    fn test_filter_operator_gt_serialization() {
        let op = FilterOperator::Gt;
        let json = serde_json::to_string(&op).unwrap();
        assert_eq!(json, "\"gt\"");
    }

    #[test]
    fn test_filter_operator_gte_serialization() {
        let op = FilterOperator::Gte;
        let json = serde_json::to_string(&op).unwrap();
        assert_eq!(json, "\"gte\"");
    }

    #[test]
    fn test_filter_operator_lt_serialization() {
        let op = FilterOperator::Lt;
        let json = serde_json::to_string(&op).unwrap();
        assert_eq!(json, "\"lt\"");
    }

    #[test]
    fn test_filter_operator_lte_serialization() {
        let op = FilterOperator::Lte;
        let json = serde_json::to_string(&op).unwrap();
        assert_eq!(json, "\"lte\"");
    }

    #[test]
    fn test_filter_operator_contains_serialization() {
        let op = FilterOperator::Contains;
        let json = serde_json::to_string(&op).unwrap();
        assert_eq!(json, "\"contains\"");
    }

    #[test]
    fn test_filter_operator_not_contains_serialization() {
        let op = FilterOperator::NotContains;
        let json = serde_json::to_string(&op).unwrap();
        assert_eq!(json, "\"not_contains\"");
    }

    #[test]
    fn test_filter_operator_in_serialization() {
        let op = FilterOperator::In;
        let json = serde_json::to_string(&op).unwrap();
        assert_eq!(json, "\"in\"");
    }

    #[test]
    fn test_filter_operator_not_in_serialization() {
        let op = FilterOperator::NotIn;
        let json = serde_json::to_string(&op).unwrap();
        assert_eq!(json, "\"not_in\"");
    }

    #[test]
    fn test_filter_operator_starts_with_serialization() {
        let op = FilterOperator::StartsWith;
        let json = serde_json::to_string(&op).unwrap();
        assert_eq!(json, "\"starts_with\"");
    }

    #[test]
    fn test_filter_operator_ends_with_serialization() {
        let op = FilterOperator::EndsWith;
        let json = serde_json::to_string(&op).unwrap();
        assert_eq!(json, "\"ends_with\"");
    }

    #[test]
    fn test_filter_operator_regex_serialization() {
        let op = FilterOperator::Regex;
        let json = serde_json::to_string(&op).unwrap();
        assert_eq!(json, "\"regex\"");
    }

    #[test]
    fn test_filter_operator_is_null_serialization() {
        let op = FilterOperator::IsNull;
        let json = serde_json::to_string(&op).unwrap();
        assert_eq!(json, "\"is_null\"");
    }

    #[test]
    fn test_filter_operator_is_not_null_serialization() {
        let op = FilterOperator::IsNotNull;
        let json = serde_json::to_string(&op).unwrap();
        assert_eq!(json, "\"is_not_null\"");
    }

    #[test]
    fn test_filter_operator_clone() {
        let op = FilterOperator::Contains;
        let cloned = op.clone();
        let json1 = serde_json::to_string(&op).unwrap();
        let json2 = serde_json::to_string(&cloned).unwrap();
        assert_eq!(json1, json2);
    }

    #[test]
    fn test_filter_operator_deserialization() {
        let eq: FilterOperator = serde_json::from_str("\"eq\"").unwrap();
        let contains: FilterOperator = serde_json::from_str("\"contains\"").unwrap();
        let is_null: FilterOperator = serde_json::from_str("\"is_null\"").unwrap();

        assert!(matches!(eq, FilterOperator::Eq));
        assert!(matches!(contains, FilterOperator::Contains));
        assert!(matches!(is_null, FilterOperator::IsNull));
    }
}
